use base64::Engine;
use dioxus::prelude::*;
use kortex_core::{ArtifactDescriptor, Runtime, VerifiedTool, dispatcher::Clock};
use kortex_runtime_native::verify_package;
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
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
        tokio::time::sleep(std::time::Duration::from_millis(
            deadline.saturating_sub(kortex_runtime_native::permissions::now_ms()),
        ))
        .await;
    }
}
type HostRuntime = Runtime<HostedNativeEngine, NativeClock>;
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
    let checked = kortex_verify::Descriptor::parse(
        &serde_json::to_vec(&proof).map_err(|_| "Descripteur invalide")?,
    )
    .map_err(|_| "Descripteur invalide")?;
    if keys.get("root").is_some() {
        let bundle = serde_json::from_value(keys).map_err(|_| "Confiance hôte invalide")?;
        return kortex_runtime_native::verify_package_with_trust(
            bytes,
            &checked,
            &bundle,
            kortex_runtime_native::permissions::now_ms(),
        )
        .map_err(|e| format!("Paquet refusé : {e:?}"));
    }
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
use kortex_capabilities::policy::{Context, Duration};
use kortex_runtime_native::{
    HostedNativeEngine,
    network::NativeNetwork,
    permissions::{CapabilityInbox, NativeHost, NativeServices},
};
use kortex_sdk::CapabilityRequest;
#[derive(Clone, Default)]
struct View {
    ui: Option<Value>,
    active: bool,
    result: String,
}
struct Model {
    runtime: HostRuntime,
    inbox: CapabilityInbox,
    services: Option<NativeServices>,
    id: Option<u64>,
    ui: Option<Value>,
    network: Vec<String>,
}
impl Model {
    fn new(inbox: CapabilityInbox) -> Self {
        Self {
            runtime: HostRuntime::new(HostedNativeEngine::new(inbox.clone()), NativeClock),
            inbox,
            services: None,
            id: None,
            ui: None,
            network: Vec::new(),
        }
    }
    fn view(&self, result: String) -> View {
        View {
            ui: self.ui.clone(),
            active: self.id.is_some(),
            result,
        }
    }
    async fn close(&mut self) {
        if let Some(id) = self.id.take() {
            if let Ok(context) = self.runtime.capability_context(id) {
                if let Some(services) = &mut self.services {
                    services.revoke(&context)
                }
            }
            let _ = self.runtime.shutdown(id).await;
        }
        self.services = None;
        self.ui = None;
        while self.inbox.pop().is_some() {}
    }
    async fn open(&mut self, path: String, trust: String) -> Result<Vec<String>, String> {
        self.close().await;
        let tool =
            kortex_runtime_native::permissions::blocking(move |_| open_verified(&path, &trust))
                .await
                .map_err(|_| "Lecture indisponible")??;
        let names = tool
            .contract()
            .actions
            .iter()
            .map(|a| a.id.clone())
            .collect();
        let host = NativeHost::app_data(
            NativeNetwork::new(self.network.clone()).map_err(|_| "Configuration réseau refusée")?,
        )
        .map_err(|_| "Stockage local indisponible")?;
        let mut services = NativeServices::new(host);
        let id = self
            .runtime
            .load(tool)
            .await
            .map_err(|e| format!("{e:?}"))?;
        self.id = Some(id);
        let context = self
            .runtime
            .prepare_capabilities(id, &mut services.gateway)
            .map_err(|e| format!("{e:?}"))?;
        services.gateway.policy.enable_prompting(&context);
        self.services = Some(services);
        match self.runtime.initialize(id).await {
            Ok(reply) => {
                self.ui = reply.data.map(Value::Object);
                Ok(names)
            }
            Err(e) => {
                self.close().await;
                Err(format!("{e:?}"))
            }
        }
    }
    async fn refresh(&mut self) -> Result<(), String> {
        let id = self.id.ok_or("Outil fermé")?;
        let reply = self
            .runtime
            .render(id)
            .await
            .map_err(|e| format!("{e:?}"))?;
        if !reply.ok {
            return Err(reply
                .error
                .map(|e| e.code)
                .unwrap_or("INVALID_REPLY".into()));
        }
        self.ui = reply.data.map(Value::Object);
        Ok(())
    }
    async fn action(&mut self, name: &str, input: &str) -> Result<String, String> {
        let value = serde_json::from_str(input).map_err(|_| "JSON invalide")?;
        let id = self.id.ok_or("Outil fermé")?;
        let reply = self
            .runtime
            .invoke_action(id, name, value)
            .await
            .map_err(|e| format!("{e:?}"))?;
        if !reply.ok {
            return Err(reply
                .error
                .map(|e| e.code)
                .unwrap_or("INVALID_REPLY".into()));
        }
        self.refresh().await?;
        Ok(serde_json::to_string(&reply.data).unwrap_or_default())
    }
    async fn resolve(
        &mut self,
        request: CapabilityRequest,
        allow: bool,
        duration: Duration,
    ) -> Result<String, String> {
        self.resolve_decision(request, Some((allow, duration)))
            .await
    }
    async fn resolve_decision(
        &mut self,
        request: CapabilityRequest,
        decision: Option<(bool, Duration)>,
    ) -> Result<String, String> {
        let id = self.id.ok_or("Outil fermé")?;
        let context: Context = self
            .runtime
            .capability_context(id)
            .map_err(|e| format!("{e:?}"))?;
        let services = self.services.as_mut().ok_or("Hôte indisponible")?;
        let outcome = if let Some((allow, duration)) = decision {
            services.fulfill(&context, &request, allow, duration).await
        } else {
            services.execute_existing(&context, &request).await
        };
        let message = outcome
            .as_ref()
            .map(|_| "Autorisation traitée".to_string())
            .unwrap_or_else(|e| e.code.clone());
        self.inbox.complete(&request.request_id);
        self.runtime
            .dispatch_capability_result(id, request.request_id, outcome)
            .await
            .map_err(|e| format!("{e:?}"))?;
        self.refresh().await?;
        Ok(message)
    }
    async fn recover_if_fatal(&mut self) {
        if self
            .id
            .is_some_and(|id| self.runtime.instance(id).is_none())
        {
            eprintln!("kortex desktop: failed instance released");
            self.close().await;
        }
    }
}
#[component]
pub fn App() -> Element {
    let inbox = use_hook(CapabilityInbox::default);
    let queue = inbox.clone();
    let model = use_hook(move || Rc::new(RefCell::new(Model::new(inbox))));
    let mut path = use_signal(String::new);
    let mut trust = use_signal(String::new);
    let mut action = use_signal(String::new);
    let mut input = use_signal(|| "{}".to_string());
    let mut view = use_signal(View::default);
    let mut busy = use_signal(|| false);
    let mut consent = use_signal(|| false);
    let mut pending = use_signal(|| None::<CapabilityRequest>);
    let startup = use_hook(|| std::env::args().skip(1).collect::<Vec<_>>());
    let config_model = model.clone();
    use_effect(move || {
        if startup.len() >= 2 {
            path.set(startup[0].clone());
            trust.set(startup[1].clone());
        }
        if let Some(file) = startup.get(2) {
            let origins = std::fs::read(file)
                .ok()
                .and_then(|v| serde_json::from_slice::<Value>(&v).ok())
                .and_then(|v| v["network_origins"].as_array().cloned())
                .and_then(|a| {
                    a.into_iter()
                        .map(|v| v.as_str().map(str::to_string))
                        .collect::<Option<Vec<_>>>()
                });
            if let Some(origins) = origins {
                config_model.borrow_mut().network = origins
            } else {
                view.set(View {
                    result: "Configuration hôte invalide".into(),
                    ..View::default()
                });
            }
        }
    });
    let automatic_model = model.clone();
    use_future(move || {
        let queue = queue.clone();
        let model = automatic_model.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if view.peek().active && !*busy.peek() && pending.peek().is_none() {
                    if let Some(request) = queue.pop() {
                        let automatic = {
                            let m = model.borrow();
                            match (m.id,&m.services,kortex_capabilities::Capability::try_from(request.capability.as_str()),kortex_runtime_native::permissions::resource(&request)){(Some(id),Some(services),Ok(cap),Ok(resource))=>m.runtime.capability_context(id).is_ok_and(|ctx|{let d=services.gateway.policy.consent_decision(&ctx,cap,&resource,kortex_runtime_native::permissions::now_ms());matches!(d,kortex_capabilities::policy::Decision::Deny)||!cap.needs_gesture()&&matches!(d,kortex_capabilities::policy::Decision::AllowOnce|kortex_capabilities::policy::Decision::AllowSession)}),_=>true}
                        };
                        if automatic {
                            busy.set(true);
                            let mut m = model.borrow_mut();
                            let result = m.resolve_decision(request, None).await;
                            m.recover_if_fatal().await;
                            view.set(m.view(result.unwrap_or_else(|e| e)));
                            busy.set(false);
                        } else {
                            pending.set(Some(request));
                        }
                    }
                }
            }
        }
    });
    let opener = model.clone();
    let closer = model.clone();
    let runner = model.clone();
    let event_model = model.clone();
    let permission_model = model.clone();
    rsx! {
    style {"{kortex_ui::SHELL_CSS}"} style {{include_str!("../assets/style.css")}}
    main {class:"desktop-shell",h1 {"Kortex Desktop — expérimental"}p {"Exécution locale Wasmtime. Chaque accès sensible exige votre accord explicite."}
    label {r#for:"package","Paquet .tool signé"}input {id:"package",value:"{path}",disabled:*busy.read(),oninput:move|e|path.set(e.value())}
    label {r#for:"trust","Clés publiques hôte (JSON)"}input {id:"trust",value:"{trust}",disabled:*busy.read(),oninput:move|e|trust.set(e.value())}
    button {disabled:*busy.read()||view.read().active,onclick:move |_|consent.set(true),"Ouvrir"}
    if *consent.read(){section {role:"dialog",aria_label:"Permissions de l’outil",h2 {"Permissions de l’outil"}p {"L’outil s’ouvre sans autorisation. Ses demandes sensibles seront affichées séparément."}
    button {onclick:move |_|consent.set(false),"Annuler"}
    button {onclick:move |_|{consent.set(false);let model=opener.clone();let path=path.read().clone();let trust=trust.read().clone();spawn(async move{busy.set(true);pending.set(None);let mut m=model.borrow_mut();let result=m.open(path,trust).await;match result{Ok(names)=>{action.set(names.first().cloned().unwrap_or_default());view.set(m.view("Prêt".into()));},Err(error)=>{m.close().await;view.set(m.view(error));}}busy.set(false);});},"Continuer sans accès sensibles"}
    }}
    button {disabled:*busy.read()||!view.read().active,onclick:move |_|{let model=closer.clone();spawn(async move{busy.set(true);pending.set(None);let mut m=model.borrow_mut();m.close().await;view.set(m.view("Fermé".into()));busy.set(false);});},"Fermer"}
    if let Some(request)=pending.read().clone(){section {role:"dialog",aria_label:"Demande d’autorisation",h2 {"Demande d’autorisation"}p {"{request.capability}"}p {"Portée : " {kortex_runtime_native::permissions::resource(&request).and_then(|r|r.scope(kortex_capabilities::Capability::try_from(request.capability.as_str())?)).unwrap_or_else(|_|"Demande invalide".into())}}p {"Autorisation limitée à cette instance et à cinq minutes maximum."}
    for (allow,duration,label) in [(false,Duration::Once,"Refuser"),(true,Duration::Once,"Autoriser une fois"),(true,Duration::Session,"Autoriser pour la session")] {button {disabled:*busy.read(),onclick:{let model=permission_model.clone();let request=request.clone();move |_|{let model=model.clone();let request=request.clone();spawn(async move{busy.set(true);pending.set(None);let mut m=model.borrow_mut();let result=m.resolve(request,allow,duration).await;m.recover_if_fatal().await;view.set(m.view(result.unwrap_or_else(|e|e)));busy.set(false);});}},"{label}"}}
    }}
    if let Some(value)=view.read().ui.clone(){kortex_ui::UiBoundary {value,on_event:move|event:kortex_ui::events::UiEvent|{let model=event_model.clone();spawn(async move{if *busy.peek(){return}busy.set(true);let mut m=model.borrow_mut();let result=if let Some(id)=m.id{match m.runtime.dispatch_event(id,event.payload()).await{Ok(reply) if reply.ok=>m.refresh().await.map(|_|"Prêt".into()),Ok(reply)=>Err(reply.error.map(|e|e.code).unwrap_or("INVALID_REPLY".into())),Err(e)=>Err(format!("{e:?}"))}}else{Err("Outil fermé".into())};m.recover_if_fatal().await;view.set(m.view(result.unwrap_or_else(|e|e)));busy.set(false);});}}}
    if view.read().active {label {r#for:"action","Action"}input {id:"action",value:"{action}",oninput:move|e|action.set(e.value())}label {r#for:"input","Paramètres JSON"}textarea {id:"input",value:"{input}",oninput:move|e|input.set(e.value())}
    button {disabled:*busy.read(),onclick:move |_|{let model=runner.clone();let name=action.read().clone();let value=input.read().clone();spawn(async move{busy.set(true);let mut m=model.borrow_mut();let result=m.action(&name,&value).await;m.recover_if_fatal().await;view.set(m.view(result.unwrap_or_else(|e|e)));busy.set(false);});},"Exécuter l’action"}}
    output {aria_label:"Résultat",role:"status","{view.read().result}"}
    }
    }
}
