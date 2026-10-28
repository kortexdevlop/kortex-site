use crate::{
    account::{self, View},
    config::Config,
};
use dioxus::prelude::*;
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
fn run<F>(
    c: Config,
    f: F,
    mut view: Signal<View>,
    mut busy: Signal<bool>,
    mut notice: Signal<String>,
    preferences: EventHandler<Value>,
) where
    F: FnOnce(Config) -> Result<View, String> + Send + 'static,
{
    busy.set(true);
    spawn(async move {
        let result = tokio::task::spawn_blocking(move || f(c)).await;
        match result {
            Ok(Ok(value)) => {
                preferences.call(value.preferences.clone());
                view.set(value);
                notice.set(String::new())
            }
            Ok(Err(error)) => notice.set(error),
            Err(_) => notice.set("Le compte n’a pas pu être chargé.".into()),
        }
        busy.set(false);
    });
}
#[component]
pub fn AccountPanel(
    config: Config,
    tools: Vec<(String, String)>,
    on_preferences: EventHandler<Value>,
) -> Element {
    let mut view = use_signal(View::default);
    let mut busy = use_signal(|| false);
    let mut notice = use_signal(String::new);
    let mut chosen = use_signal(String::new);
    let cancel = use_hook(|| Arc::new(AtomicBool::new(false)));
    let dropped = cancel.clone();
    use_drop(move || dropped.store(true, Ordering::SeqCst));
    use_effect({
        let c = config.clone();
        move || {
            run(
                c.clone(),
                |c| account::view(&c),
                view,
                busy,
                notice,
                on_preferences,
            )
        }
    });
    let login = config.clone();
    let flag = cancel.clone();
    let refresh = config.clone();
    let optin = config.clone();
    let logout = config.clone();
    let add = config.clone();
    let snapshot = view.read().clone();
    rsx! {h1{"Mon espace"}
    p{class:"muted","Vos outils locaux restent disponibles sans compte et hors connexion."}
    if !notice.read().is_empty(){p{class:"banner",role:"status","{notice}"}}
    if !snapshot.message.is_empty(){p{class:"banner",role:"status","{snapshot.message}"}}
    if !snapshot.connected{
     div{class:"empty",h2{"Connecter mon compte"}p{"La connexion s’ouvre dans votre navigateur habituel. Aucun mot de passe n’est saisi dans l’application."}
     button{disabled:*busy.read()||config.offline,onclick:move |_|{let flag=flag.clone();flag.store(false,Ordering::SeqCst);notice.set("Poursuivez dans votre navigateur puis autorisez cet appareil.".into());run(login.clone(),move|c|{let pending=account::start(&c)?;webbrowser::open(&pending.authorization_url).map_err(|_|"Le navigateur n’a pas pu être ouvert.".to_string())?;account::complete(&c,pending,flag)},view,busy,notice,on_preferences)},"Se connecter dans le navigateur"}
     if *busy.read(){button{onclick:move |_|{cancel.store(true,Ordering::SeqCst);notice.set("Annulation en cours…".into())},"Annuler la connexion"}}
     }
    }else{
     div{class:"empty",h2{"Compte connecté"}p{"{snapshot.user}"}
     label{input{r#type:"checkbox",checked:snapshot.enabled,disabled:*busy.read()||config.offline,onchange:move|event|{let enabled=event.checked();run(optin.clone(),move|c|account::synchronize(&c,Some(enabled)),view,busy,notice,on_preferences)}}" Synchroniser mes favoris et réglages sur cet appareil"}
     p{class:"muted","Aucun fichier n’est envoyé. Ajouter un favori ne télécharge pas automatiquement l’outil."}
     button{disabled:*busy.read()||config.offline,onclick:move |_|run(refresh.clone(),|c|account::synchronize(&c,None),view,busy,notice,on_preferences),"Actualiser la synchronisation"}
     button{disabled:*busy.read(),onclick:move |_|{let c=logout.clone();busy.set(true);spawn(async move{let result=tokio::task::spawn_blocking(move||account::logout(&c)).await;match result{Ok(Ok(message))=>{view.set(View::default());on_preferences.call(Value::Null);notice.set(message)},_=>notice.set("La déconnexion n’a pas abouti.".into())}busy.set(false);});},"Se déconnecter"}
     }
     if snapshot.enabled{
      h2{"Réglages partagés"}
      {[ ("theme","Thème",vec![("light","Clair"),("dark","Sombre")]),("density","Densité",vec![("simple","Simple"),("medium","Moyenne"),("pro","Compacte")]),("type","Taille du texte",vec![("standard","Standard"),("large","Grande")]),("lang","Langue préférée",vec![("fr","Français"),("en","English")]) ].into_iter().map(|(key,label,options)|{let update=config.clone();let value=snapshot.preferences[key].as_str().unwrap_or("").to_owned();rsx!{label{r#for:"account-{key}","{label}"}select{id:"account-{key}",value,disabled:*busy.read()||config.offline,onchange:move|event|{let value=event.value();run(update.clone(),move|c|account::preference(&c,key,&value),view,busy,notice,on_preferences)},for (value,label) in options{option{value,"{label}"}}}}})}
      h2{"Favoris partagés"}
      if snapshot.favorites.is_empty(){p{class:"muted","Aucun favori synchronisé."}}
      {snapshot.favorites.iter().map(|id|{let remove=config.clone();let selected=id.clone();rsx!{article{class:"account-favorite",strong{"{id}"}button{disabled:*busy.read()||config.offline,onclick:move |_|{let selected=selected.clone();run(remove.clone(),move|c|account::favorite(&c,&selected,false),view,busy,notice,on_preferences)},"Retirer le favori"}}}})}
      label{r#for:"account-tool","Ajouter un outil aux favoris"}select{id:"account-tool",value:"{chosen}",onchange:move|event|chosen.set(event.value()),option{value:"","Choisir un outil"}for (id,name) in tools{option{value:id,"{name}"}}}
      button{disabled:*busy.read()||config.offline||chosen.read().is_empty(),onclick:move |_|{let id=chosen.read().clone();run(add.clone(),move|c|account::favorite(&c,&id,true),view,busy,notice,on_preferences)},"Ajouter le favori"}
     }
    }
    }
}
