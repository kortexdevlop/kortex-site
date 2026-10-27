use base64::Engine;
use dioxus::prelude::*;
use kortex_core::{ArtifactDescriptor, Runtime, VerifiedTool, dispatcher::Clock};
use kortex_runtime_native::{NativeGuestEngine, verify_package};
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    future::poll_fn,
    rc::Rc,
    task::Poll,
    time::{SystemTime, UNIX_EPOCH},
};
struct NativeClock;
impl Clock for NativeClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    async fn wait_until(&self, deadline: u64) {
        poll_fn(|cx| {
            if self.now_ms() >= deadline {
                Poll::Ready(())
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .await
    }
}
type HostRuntime = Runtime<NativeGuestEngine, NativeClock>;
pub fn open_verified(path: &str, trust_path: &str) -> Result<VerifiedTool, String> {
    if std::fs::metadata(path)
        .map_err(|_| "Paquet illisible")?
        .len()
        > 25 * 1024 * 1024
    {
        return Err("Paquet trop volumineux".into());
    }
    let bytes = std::fs::read(path).map_err(|_| "Paquet illisible")?;
    let proof: Value = serde_json::from_slice(
        &std::fs::read(format!("{path}.json")).map_err(|_| "Descripteur signé manquant")?,
    )
    .map_err(|_| "Descripteur invalide")?;
    let keys: Value = serde_json::from_slice(
        &std::fs::read(trust_path).map_err(|_| "Clés publiques hôte manquantes")?,
    )
    .map_err(|_| "Clés invalides")?;
    let mut trusted = BTreeMap::new();
    for (id, value) in keys.as_object().ok_or("Clés invalides")? {
        let raw = base64::engine::general_purpose::STANDARD
            .decode(value.as_str().ok_or("Clé invalide")?)
            .map_err(|_| "Clé invalide")?;
        trusted.insert(id.clone(), raw.try_into().map_err(|_| "Clé invalide")?);
    }
    let text = |field: &str| -> Result<String, String> {
        Ok(proof[field].as_str().ok_or("Descripteur incomplet")?.into())
    };
    let signature = base64::engine::general_purpose::STANDARD
        .decode(text("signature")?)
        .map_err(|_| "Signature invalide")?
        .try_into()
        .map_err(|_| "Signature invalide")?;
    let descriptor = ArtifactDescriptor {
        schema_version: proof["schema_version"]
            .as_u64()
            .and_then(|v| u8::try_from(v).ok())
            .ok_or("Version de descripteur invalide")?,
        tool_id: text("tool_id")?,
        version: text("version")?,
        abi_version: text("abi_version")?,
        sha256: text("sha256")?,
        signing_key_id: text("signing_key_id")?,
        signature,
    };
    verify_package(bytes, &descriptor, &trusted).map_err(|e| format!("Paquet refusé : {e:?}"))
}
#[component]
pub fn App() -> Element {
    let runtime = use_hook(|| {
        Rc::new(RefCell::new(HostRuntime::new(
            NativeGuestEngine,
            NativeClock,
        )))
    });
    let mut path = use_signal(String::new);
    let mut trust = use_signal(String::new);
    let mut ui = use_signal(|| None::<Value>);
    let mut instance = use_signal(|| None::<u64>);
    let mut actions = use_signal(Vec::<String>::new);
    let mut action = use_signal(String::new);
    let mut input = use_signal(|| "{}".to_string());
    let mut result = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut consent = use_signal(|| false);
    let opener = runtime.clone();
    let closer = runtime.clone();
    let runner = runtime.clone();
    let event_runtime = runtime.clone();
    let startup = use_hook(|| std::env::args().skip(1).collect::<Vec<_>>());
    use_effect(move || {
        if startup.len() >= 2 {
            path.set(startup[0].clone());
            trust.set(startup[1].clone());
        }
    });
    rsx! {style {"{kortex_ui::SHELL_CSS}"}style {{include_str!("../assets/style.css")}}
    main {class:"desktop-shell",h1 {"Kortex Desktop — expérimental"}p {"Exécution locale Wasmtime. Les accès sensibles restent refusés jusqu’au raccordement des capacités natives."}
    label {r#for:"package","Paquet .tool signé"}input {id:"package",value:"{path}",oninput:move|e|path.set(e.value())}
    label {r#for:"trust","Clés publiques hôte (JSON)"}input {id:"trust",value:"{trust}",oninput:move|e|trust.set(e.value())}
    button {disabled:*busy.read()||instance.read().is_some(),onclick:move |_|consent.set(true),"Ouvrir"}
    if *consent.read(){section {role:"dialog",aria_label:"Permissions de l’outil",h2 {"Permissions de l’outil"}p {"Cet hôte refuse les accès au réseau, aux fichiers et au presse-papiers. Vous pouvez ouvrir l’outil dans ce mode limité."}
    button {onclick:move |_|consent.set(false),"Annuler"}
    button {onclick:move |_|{consent.set(false);let rt=opener.clone();spawn(async move{busy.set(true);let outcome=async{let tool=open_verified(&path.read(),&trust.read())?;let names=tool.contract().actions.iter().map(|a|a.id.clone()).collect::<Vec<_>>();let mut rt=rt.try_borrow_mut().map_err(|_|"Opération en cours".to_string())?;let id=rt.load(tool).await.map_err(|e|format!("{e:?}"))?;let render=match rt.initialize(id).await{Ok(render)=>render,Err(e)=>{let _=rt.shutdown(id).await;return Err(format!("{e:?}"));}};actions.set(names.clone());if let Some(name)=names.first(){action.set(name.clone())}instance.set(Some(id));ui.set(render.data.map(Value::Object));Ok::<(),String>(())}.await;result.set(outcome.err().unwrap_or_else(||"Prêt".into()));busy.set(false);});},"Continuer sans accès sensibles"}
    }}
    button {disabled:*busy.read()||instance.read().is_none(),onclick:move |_|{let rt=closer.clone();spawn(async move{if let Some(id)=*instance.read(){if let Ok(mut rt)=rt.try_borrow_mut(){let _=rt.shutdown(id).await;}}instance.set(None);ui.set(None);actions.set(Vec::new());result.set("Fermé".into());});},"Fermer"}
    if let Some(value)=ui.read().clone(){kortex_ui::UiBoundary {value,on_event:move|event:kortex_ui::events::UiEvent|{let rt=event_runtime.clone();spawn(async move{if let Some(id)=*instance.read(){if let Ok(mut rt)=rt.try_borrow_mut(){match rt.dispatch_event(id,event.payload()).await{Ok(reply)=>{if !reply.ok{result.set(format!("{:?}",reply.error));}},Err(e)=>result.set(format!("{e:?}"))}}}});}}}
    if instance.read().is_some(){label {r#for:"action","Action"}input {id:"action",value:"{action}",oninput:move|e|action.set(e.value())}label {r#for:"input","Paramètres JSON"}textarea {id:"input",value:"{input}",oninput:move|e|input.set(e.value())}
    button {disabled:*busy.read(),onclick:move |_|{let rt=runner.clone();spawn(async move{busy.set(true);let outcome=async{let value:Value=serde_json::from_str(&input.read()).map_err(|_|"JSON invalide".to_string())?;let id=instance.read().ok_or("Outil fermé")?;let mut rt=rt.try_borrow_mut().map_err(|_|"Opération en cours".to_string())?;let reply=rt.invoke_action(id,&action.read(),value).await.map_err(|e|format!("{e:?}"))?;Ok::<String,String>(serde_json::to_string(&reply.data).unwrap_or_else(|_|"Erreur".into()))}.await;result.set(outcome.unwrap_or_else(|e|e));busy.set(false);});},"Exécuter l’action"}}
    output {aria_label:"Résultat",role:"status","{result}"}
    }}
}
