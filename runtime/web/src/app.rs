use dioxus::prelude::*;
use kortex_ui::UiBoundary;
use serde_json::{Value, json};
/// Public host configuration; never includes credentials or guest-selected origins.
#[derive(Clone, Debug)]
pub struct RuntimeWebConfig {
    pub api_origin: String,
    pub allowed_parent_origin: String,
    pub tool_id: String,
    pub tool_version: String,
}
impl RuntimeWebConfig {
    pub fn validate(&self) -> bool {
        fn origin(s: &str) -> bool {
            url::Url::parse(s).is_ok_and(|u| {
                u.origin().ascii_serialization() == s
                    && (u.scheme() == "https"
                        || (u.scheme() == "http"
                            && matches!(u.host_str(), Some("localhost" | "127.0.0.1"))))
            })
        }
        origin(&self.api_origin)
            && origin(&self.allowed_parent_origin)
            && !self.tool_id.is_empty()
            && !self.tool_version.is_empty()
    }
}
fn command(message: Value) {
    spawn(async move {
        let eval = document::eval(
            "const message=await dioxus.recv(); if(window.kortexHost) await window.kortexHost.command(message);",
        );
        if eval.send(message).is_ok() {
            let _ = eval.await;
        }
    });
}
#[component]
pub fn App() -> Element {
    let mut snapshot = use_signal(|| json!({"status":"loading"}));
    let mut mode = use_signal(|| "standard".to_string());
    use_future(move || async move {
        let mut stream = document::eval(
            r#"
   const send=()=>dioxus.send(window.kortexHost?.snapshot() ?? {status:navigator.onLine?'unavailable':'offline'});
   window.addEventListener('kortex:host',send); window.addEventListener('online',send);window.addEventListener('offline',send);send();
   await new Promise(()=>{});
  "#,
        );
        while let Ok(value) = stream.recv::<Value>().await {
            let invalid = value.get("config").is_some_and(|c| {
                let config = RuntimeWebConfig {
                    api_origin: c["api_origin"].as_str().unwrap_or("").into(),
                    allowed_parent_origin: c["allowed_parent_origin"].as_str().unwrap_or("").into(),
                    tool_id: c["tool_id"].as_str().unwrap_or("").into(),
                    tool_version: c["tool_version"].as_str().unwrap_or("").into(),
                };
                !config.validate()
            });
            snapshot.set(if invalid {
                json!({"status":"error"})
            } else {
                value
            });
        }
    });
    let current = snapshot.read().clone();
    let status = current["status"].as_str().unwrap_or("error");
    let (title, message) = match status {
        "loading" => ("Chargement", "Préparation de l’hôte…"),
        "ready" => ("Prêt", "L’outil vérifié est prêt."),
        "denied" => (
            "Accès refusé",
            "L’autorisation demandée n’a pas été accordée.",
        ),
        "offline" => (
            "Hors connexion",
            "Certaines fonctions nécessitent une connexion.",
        ),
        "unavailable" => ("Indisponible", "Aucun outil signé n’est chargé."),
        _ => ("Erreur", "L’outil n’a pas pu être exécuté."),
    };
    rsx! {
    document::Script {r#type:"module",src:"/adapter/host.js"} style {"{kortex_ui::SHELL_CSS}"}style {{include_str!("../assets/style.css")}}
    main {class:"runtime-shell","data-mode":"{mode}","data-status":status,
     header {class:"runtime-header",h1 {"Kortex"}label {"Affichage " select {aria_label:"Mode d’affichage",value:"{mode}",onchange:move |e|mode.set(e.value()),option {value:"standard","Standard"}option {value:"compact","Compact"}option {value:"contrast","Contraste élevé"}}}}
     section {class:"runtime-status",role:if status=="error"||status=="denied" {"alert"}else{"status"},h2 {"{title}"}p {"{message}"}}
     if let Some(ui)=current.get("ui"){UiBoundary {value:ui.clone(),on_event:move |event:kortex_ui::events::UiEvent|command(json!({"kind":"ui.event","payload":event.payload()}))}}
     if let Some(prompts)=current["pending"].as_array(){for prompt in prompts {section {class:"runtime-status",aria_label:"Demande d’autorisation",h2 {"Autorisation demandée"}p {{prompt["tool_id"].as_str().unwrap_or("")} " — " {prompt["capability"].as_str().unwrap_or("")}}p {{prompt["scope"].as_str().unwrap_or("")}}p {"Accès local, limité à cette session. Aucun secret ne doit être enregistré."}div {class:"runtime-actions",for (decision,label) in [("once","Autoriser une fois"),("session","Autoriser pour la session"),("deny","Refuser")] {button {r#type:"button","data-host-approval":prompt["id"].as_str().unwrap_or(""),"data-host-decision":decision,"{label}"}}}}}}
        if current.get("result").is_some(){output {aria_label:"Résultat", {current["result"].to_string()}}}
     div {class:"runtime-actions",button {r#type:"button",onclick:move |_|command(json!({"kind":"stop"})),"Fermer l’outil"}button {r#type:"button",onclick:move |_|command(json!({"kind":"revoke"})),"Révoquer les autorisations"}}
    }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_has_only_fixed_public_origins() {
        let mut c = RuntimeWebConfig {
            api_origin: "https://api.example.test".into(),
            allowed_parent_origin: "https://www.example.test".into(),
            tool_id: "demo".into(),
            tool_version: "1.0.0".into(),
        };
        assert!(c.validate());
        c.api_origin = "https://api.example.test/path".into();
        assert!(!c.validate());
    }
}
