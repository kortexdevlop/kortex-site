use crate::{config::Config, workflows};
use dioxus::prelude::*;
use kortex_workflows::runner::Cancellation;
use std::sync::{Arc, Mutex};
#[component]
pub fn WorkflowView(config: Config) -> Element {
    let mut text = use_signal(String::new);
    let mut approved = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut result = use_signal(String::new);
    let recovery_config = config.clone();
    let mut notice = use_signal(move || {
        if workflows::recover_last_run(&recovery_config).unwrap_or(false) {
            "Le parcours précédent a été interrompu à la fermeture. Aucun redémarrage automatique."
                .into()
        } else {
            String::new()
        }
    });
    let mut cloud = use_signal(|| None::<(u64, String)>);
    let mut file = use_signal(|| None::<dioxus::html::FileData>);
    let mut file_node = use_signal(String::new);
    let mut file_consent = use_signal(|| false);
    let save_config = config.clone();
    let load_config = config.clone();
    let read_config = config.clone();
    let sync_config = config.clone();
    let progress = use_hook(|| Arc::new(Mutex::new(Vec::<String>::new())));
    let mut lines = use_signal(Vec::<String>::new);
    let current = use_hook(|| Arc::new(Mutex::new(Cancellation::default())));
    let permissions = use_hook(crate::workflow_permissions::Permissions::interactive);
    let mut pending = use_signal(|| None::<crate::workflow_permissions::Prompt>);
    let mut resource = use_signal(String::new);
    let questions = permissions.clone();
    let poll = progress.clone();
    use_future(move || {
        let p = poll.clone();
        let questions = questions.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let next = p.lock().map(|v| v.clone()).unwrap_or_default();
                if *lines.peek() != next {
                    lines.set(next)
                }
                let question = questions.pending();
                if *pending.peek() != question {
                    if let Some(q) = &question {
                        resource.set(q.resource.clone())
                    }
                    pending.set(question)
                }
            }
        }
    });
    let cleanup = current.clone();
    use_drop(move || {
        if let Ok(token) = cleanup.lock() {
            token.cancel()
        }
    });
    let stop = current.clone();
    let prompt = pending.read().clone();
    let permission_dialog=prompt.map(|q|{let yes=permissions.clone();let no=permissions.clone();let yes_id=q.id.clone();let no_id=q.id.clone();let label=match q.capability.as_str(){"storage.read"=>"Lire le stockage local","storage.write"=>"Modifier le stockage local","file.pick"=>"Choisir un fichier","file.read"=>"Lire un fichier","file.save"=>"Enregistrer un fichier","network.fetch"=>"Accéder à une adresse Internet","clipboard.write"=>"Écrire dans le presse-papiers",_=>"Accès demandé"};rsx!{section{role:"alertdialog",aria_label:"Autorisation du parcours",class:"empty",h2{"{label}"}p{"Outil : {q.tool} · {q.version} — étape {q.node}"}label{r#for:"workflow-permission-resource","Ressource concernée"}input{id:"workflow-permission-resource",value:"{resource}",readonly:!q.editable,oninput:move|e|resource.set(e.value())}if q.capability=="network.fetch"{p{"L’autorisation porte sur l’origine de cette adresse ; les adresses privées restent interdites."}}p{"Autorisation pour cette exécution uniquement. Refuser arrête le parcours."}button{onclick:move |_|yes.respond(&yes_id,true,resource.read().clone()),"Autoriser cet accès"}button{onclick:move |_|no.respond(&no_id,false,resource.read().clone()),"Refuser cet accès"}}}});
    let run_permissions = permissions.clone();
    rsx! {h1{"Vos parcours"}p{class:"muted","Importez le JSON exporté depuis le Web. Les outils doivent être installés et signés. Les résultats restent sur cet appareil."}
    label{r#for:"desktop-workflow-json","Définition du parcours"}textarea{id:"desktop-workflow-json",maxlength:1048576,disabled:*busy.read(),value:"{text}",oninput:move|e|{text.set(e.value());approved.set(false)}}
    label{input{r#type:"checkbox",disabled:*busy.read(),checked:*approved.read(),onchange:move|e|approved.set(e.checked())}" J’ai vérifié les étapes et leurs paramètres et j’approuve leur exécution locale. Chaque accès sensible demandera une autorisation distincte."}
    {permission_dialog}
    label{r#for:"workflow-local-file","Fichier local pour cette exécution"}input{id:"workflow-local-file",r#type:"file",disabled:*busy.read(),onchange:move|e|{file.set(e.files().first().cloned());file_consent.set(false)}}
    label{r#for:"workflow-file-node","Identifiant de l’étape recevant le fichier"}input{id:"workflow-file-node",value:"{file_node}",disabled:*busy.read(),oninput:move|e|{file_node.set(e.value());file_consent.set(false)}}
    label{input{r#type:"checkbox",disabled:*busy.read(),checked:*file_consent.read(),onchange:move|e|file_consent.set(e.checked())}" J’autorise la lecture locale de ce fichier pour cette exécution."}
    button{disabled:*busy.read(),onclick:move |_|{let r=serde_json::from_str(&text.read()).map_err(|_|"JSON invalide".to_string()).and_then(|v|crate::workflow_store::save(&save_config,&v));notice.set(r.map(|_|"Sauvegarde locale terminée.".into()).unwrap_or_else(|e|e));},"Sauvegarder sur cet appareil"}
    button{disabled:*busy.read(),onclick:move |_|{match crate::workflow_store::load(&load_config){Ok(v)=>{text.set(serde_json::to_string_pretty(&v).unwrap_or_default());approved.set(false);notice.set("Copie locale chargée. Vérifiez avant d’exécuter.".into())},Err(e)=>notice.set(e)}},"Charger la sauvegarde locale"}
    button{disabled:*busy.read(),onclick:move |_|{let c=read_config.clone();busy.set(true);spawn(async move{match tokio::task::spawn_blocking(move||crate::account::workflow_request(&c,reqwest::Method::GET,None,None,None,None)).await{Ok(Ok(v))=>{if let (Some(revision),Some(owner))=(v["revision"].as_u64(),v["client_owner"].as_str()){cloud.set(Some((revision,owner.into())));notice.set("Liste chargée. Vous pouvez sauvegarder ce parcours ; aucun résultat n’est envoyé.".into());result.set(serde_json::to_string_pretty(&v).unwrap_or_default());}else{notice.set("Réponse invalide.".into())}},Ok(Err(e))=>notice.set(e),Err(_)=>notice.set("Synchronisation indisponible.".into())}busy.set(false);});},"Charger mes parcours du compte"}
    button{disabled:*busy.read()||cloud.read().is_none(),onclick:move |_|{let Some((revision,owner))=cloud.read().clone()else{return};let value=match serde_json::from_str(&text.read()){Ok(v)=>v,Err(_)=>{notice.set("JSON invalide.".into());return}};let c=sync_config.clone();busy.set(true);spawn(async move{match tokio::task::spawn_blocking(move||crate::workflow_store::sync(&c,&value,revision,&owner)).await{Ok(Ok(_))=>notice.set("Parcours sauvegardé dans votre compte.".into()),Ok(Err(e))=>notice.set(e),Err(_)=>notice.set("Synchronisation interrompue.".into())}cloud.set(None);busy.set(false);});},"Sauvegarder ce parcours dans mon compte"}
    button{disabled:*busy.read()||!*approved.read()||text.read().is_empty(),onclick:move |_|{let value=match serde_json::from_str(&text.read()){Ok(v)=>v,Err(_)=>{notice.set("JSON invalide.".into());return}};let token=Cancellation::default();if let Ok(mut current)=current.lock(){*current=token.clone()}let broker=run_permissions.clone();let selected_file=file.read().as_ref().map(|f|f.path());let selected_node=file_node.read().clone();let allowed_file=*file_consent.read();let c=config.clone();let p=progress.clone();if let Ok(mut log)=p.lock(){log.clear()}busy.set(true);result.set(String::new());notice.set("Exécution locale…".into());spawn(async move{match tokio::task::spawn_blocking(move||{let value=if let Some(path)=selected_file{workflows::bind_file(&value,&selected_node,&path,allowed_file,&token)?}else{value};workflows::execute_with_permissions(&c,value,true,token,p,broker)}).await{Ok(Ok(value))=>{notice.set(format!("État : {}",value["status"].as_str().unwrap_or("inconnu")));result.set(serde_json::to_string_pretty(&value).unwrap_or_default())},Ok(Err(e))=>notice.set(e),Err(_)=>notice.set("Exécution interrompue.".into())}busy.set(false);});},"Exécuter le parcours"}
    button{disabled:!*busy.read(),onclick:move |_|{if let Ok(token)=stop.lock(){token.cancel()}notice.set("Arrêt demandé. Les effets déjà réalisés restent présents.".into());},"Arrêter le parcours"}
    p{role:"status","{notice}"}ul{for line in lines.read().iter(){li{"{line}"}}}if !result.read().is_empty(){h2{"Résultats locaux"}pre{"{result}"}}
    }
}
