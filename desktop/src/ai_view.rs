use crate::{account, ai_bridge, config::Config};
use dioxus::prelude::*;
use serde_json::{Value, json};
#[component]
pub fn AiView(config: Config) -> Element {
    let mut message = use_signal(String::new);
    let mut consent = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut plan = use_signal(|| Value::Null);
    let mut states = use_signal(Vec::<String>::new);
    let mut results = use_signal(Vec::<String>::new);
    let mut notice = use_signal(String::new);
    let mut born = use_signal(|| 0u64);
    let mut plan_owner = use_signal(|| None::<String>);
    let request = config.clone();
    let execute = config.clone();
    let summary = plan.read()["summary"].as_str().unwrap_or("").to_owned();
    rsx! {h1{"Votre assistant"}p{class:"muted","L’IA propose. Vous décidez. L’exécution reste sur cet appareil."}
    label{r#for:"desktop-ai-message","Votre demande"}textarea{id:"desktop-ai-message",maxlength:1000,value:"{message}",oninput:move|e|message.set(e.value())}
    label{input{r#type:"checkbox",checked:*consent.read(),onchange:move|e|consent.set(e.checked())}" J’autorise l’envoi de cette demande sans données privées à Workers AI."}
    p{class:"muted","Connectez votre compte, puis autorisez l’IA pour cet appareil dans les réglages Web. Seuls les outils déjà installés sont proposés."}
    button{disabled:*busy.read()||!*consent.read()||config.offline||message.read().trim().is_empty(),onclick:move |_|{let c=request.clone();let text=message.read().clone();let identity=crate::secure_store::load(&c).ok().flatten().map(|s|s.user_id.clone());busy.set(true);plan.set(Value::Null);spawn(async move{match tokio::task::spawn_blocking(move||account::ai_proposal(&c,&text)).await{Ok(Ok(value)) if kortex_core::ai_bridge::bounded_plan(&value)=>{let n=value["steps"].as_array().map_or(0,Vec::len);born.set(kortex_runtime_native::permissions::now_ms());plan_owner.set(identity);states.set(vec!["À vérifier".into();n]);results.set(vec![String::new();n]);plan.set(value);notice.set("Vérifiez chaque action avant de l’approuver.".into())},Ok(Err(error))=>notice.set(error),_=>notice.set("Proposition invalide. Rien n’a été exécuté.".into())}busy.set(false);});},"Demander une proposition"}
    button{disabled:*busy.read(),onclick:move |_|{born.set(kortex_runtime_native::permissions::now_ms());plan_owner.set(None);plan.set(json!({"v":1,"summary":"Exemple local sans IA : 25 × 48","steps":[{"tool_id":"calculatrice","action_id":"multiply","input":{"a":25,"b":48},"requires_consent":true}]}));states.set(vec!["À vérifier".into()]);results.set(vec![String::new()]);notice.set("La calculatrice doit être installée. Aucun appel IA.".into());},"Essayer un exemple local"}
    if config.offline{p{class:"banner","IA indisponible hors connexion. Les outils installés et l’exemple local restent disponibles."}}
    p{class:"banner",role:"status","{notice}"}
    if !plan.read().is_null(){h2{"Propositions à vérifier"}p{"{summary}"}
    {plan.read()["steps"].as_array().cloned().unwrap_or_default().into_iter().enumerate().map(|(index,step)|{let c=execute.clone();let s=step.clone();let label=format!("{} · {}",step["tool_id"].as_str().unwrap_or(""),step["action_id"].as_str().unwrap_or(""));let input=kortex_core::ai_bridge::local_result(step["input"].clone()).unwrap_or(Value::Null).to_string();let state=states.read().get(index).cloned().unwrap_or_default();let result=results.read().get(index).cloned().unwrap_or_default();rsx!{article{class:"empty",h2{"{label}"}pre{"{input}"}p{"{state}"}
    button{disabled:*busy.read()||state!="À vérifier",onclick:move |_|{states.write()[index]="Vérification locale…".into();busy.set(true);let c=c.clone();let step=s.clone();let created=*born.read();let expected=plan_owner.read().clone();spawn(async move{let p=json!({"v":1,"summary":"Action approuvée localement","steps":[step]});match tokio::task::spawn_blocking(move||ai_bridge::execute_reviewed(&c,p,expected.as_deref(),created)).await{Ok(Ok(value))=>{states.write()[index]="Exécutée localement".into();results.write()[index]=value.to_string()},Ok(Err(error))=>states.write()[index]=error,_=>states.write()[index]="Exécution interrompue.".into()}busy.set(false);});},"Approuver l’action {index+1}"}
    button{disabled:*busy.read()||state!="À vérifier",onclick:move |_|states.write()[index]="Refusée".into(),"Refuser l’action {index+1}"}
    if !result.is_empty(){p{"Résultat local — aucun envoi au cloud"}pre{"{result}"}}}}})}}
    }
}
