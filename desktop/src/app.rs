use crate::{
    api::{Api, Query},
    config::Config,
    installer,
    launcher::LaunchPlan,
    library::{self, Library},
    manager::{self, Choice, Operation},
    models::ToolDetail,
    store::Store,
    updates,
};
use dioxus::prelude::*;
#[derive(Clone, Copy, PartialEq)]
enum Section {
    Store,
    Library,
    Updates,
    Account,
    Ai,
    Workflows,
}
fn category_label(value: &str) -> String {
    match value {
        "developer" => "Développeur",
        "security" => "Sécurité",
        "calculation" => "Calcul",
        "data" => "Données",
        "image" => "Image",
        "document" | "documents" => "Documents",
        "video-audio" | "media" => "Vidéo et audio",
        "design" => "Design",
        other => other,
    }
    .into()
}
fn load(
    api: Api,
    mut store: Signal<Store>,
    mut busy: Signal<bool>,
    mut status: Signal<String>,
    mut epoch: Signal<u64>,
    query: Query,
    cursor: Option<String>,
    reset: bool,
) {
    if *busy.peek() {
        return;
    }
    let ticket = *epoch.peek() + 1;
    epoch.set(ticket);
    busy.set(true);
    status.set(String::new());
    if reset {
        store.write().reset(query.clone());
    }
    spawn(async move {
        let result =
            tokio::task::spawn_blocking(move || api.list(&query, cursor.as_deref(), false)).await;
        if *epoch.peek() != ticket {
            return;
        }
        let result = result
            .map_err(|_| "Catalogue indisponible.".to_string())
            .and_then(|r| r.map_err(|e| e.to_string()))
            .and_then(|page| store.write().accept(page));
        status.set(result.map(|_| String::new()).unwrap_or_else(|e| e));
        busy.set(false);
    });
}
fn details(
    api: Api,
    id: String,
    mut detail: Signal<Option<ToolDetail>>,
    mut busy: Signal<bool>,
    mut status: Signal<String>,
) {
    if *busy.peek() {
        return;
    }
    busy.set(true);
    status.set(String::new());
    spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            let d = api.detail_fresh(&id)?;
            let actions = api.actions_fresh(&id, &d.tool.version)?;
            if serde_json::to_value(&actions.actions).ok()
                != serde_json::to_value(&d.contract.actions).ok()
            {
                return Err(crate::api::Error::Invalid(
                    "Actions de la fiche incohérentes.".into(),
                ));
            }
            Ok(d)
        })
        .await;
        match result {
            Ok(Ok(v)) => detail.set(Some(v)),
            Ok(Err(e)) => status.set(e.to_string()),
            Err(_) => status.set("Fiche indisponible.".into()),
        };
        busy.set(false);
    });
}
#[component]
pub fn App() -> Element {
    let config = use_hook(Config::load);
    let api = use_hook({
        let config = config.clone();
        move || {
            config
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|c| Api::new(c).map_err(|e| e.to_string()))
        }
    });
    let start_offline = config.as_ref().is_ok_and(|c| c.offline);
    let mut offline_mode = use_signal(move || start_offline);
    let mut config = config.clone();
    if let Ok(c) = &mut config {
        c.offline = *offline_mode.read();
    }
    let api = api.map(|a| a.offline_mode(*offline_mode.read()));
    let mut section = use_signal(|| Section::Store);
    let mut store = use_signal(Store::default);
    let mut busy = use_signal(|| false);
    let mut status = use_signal(String::new);
    let mut account_preferences = use_signal(|| serde_json::Value::Null);
    let epoch = use_signal(|| 0u64);
    let mut query = use_signal(String::new);
    let mut category = use_signal(String::new);
    let mut sort = use_signal(|| "recent".to_string());
    let mut detail = use_signal(|| None::<ToolDetail>);
    let mut active = use_signal(|| None::<LaunchPlan>);
    let mut library = use_signal(Library::default);
    let mut confirmation = use_signal(|| None::<Choice>);
    let available = use_signal(Vec::<updates::Update>::new);
    let checked = use_signal(|| false);
    let notice = use_signal(String::new);
    let monitor =
        use_hook(|| std::sync::Arc::new(std::sync::Mutex::new(installer::Progress::default())));
    let cancel = use_hook(|| std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)));
    let mut progress = use_signal(installer::Progress::default);
    let poll = monitor.clone();
    use_future(move || {
        let poll = poll.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                if *busy.peek() {
                    progress.set(poll.lock().unwrap().clone())
                }
            }
        }
    });
    let boot_config = config.clone();
    let boot_api = api.clone();
    use_future(move || {
        let config = boot_config.clone();
        let api = boot_api.clone();
        async move {
            busy.set(true);
            match config {
                Ok(c) => {
                    let local = c.clone();
                    if let Ok(v) = tokio::task::spawn_blocking(move || library::scan(&local)).await
                    {
                        library.set(v)
                    }
                    if c.offline {
                        status.set("Hors connexion. Votre bibliothèque reste disponible.".into())
                    } else if let Ok(api) = api {
                        let result = tokio::task::spawn_blocking(move || {
                            api.list(&Query::default(), None, false)
                        })
                        .await;
                        match result {
                            Ok(Ok(page)) => {
                                if let Err(e) = store.write().accept(page) {
                                    status.set(e)
                                }
                            }
                            Ok(Err(e)) => status.set(e.to_string()),
                            Err(_) => status.set("Catalogue indisponible.".into()),
                        }
                    }
                }
                Err(e) => status.set(e),
            }
            busy.set(false);
        }
    });
    let offline = config.as_ref().is_ok_and(|c| c.offline);
    let channel = config
        .as_ref()
        .map(|c| c.channel.as_str())
        .unwrap_or("staging");
    let header = match *section.read() {
        Section::Store => "Store",
        Section::Library => "Bibliothèque",
        Section::Updates => "Mises à jour",
        Section::Account => "Compte",
        Section::Ai => "Assistant",
        Section::Workflows => "Parcours",
    };
    let search_api = api.clone();
    let sort_api = api.clone();
    let category_api = api.clone();
    let installed = library.read().tools.clone();
    let categories: Vec<String> = store.read().known_categories.iter().cloned().collect();
    let rows = store
        .read()
        .page
        .as_ref()
        .map(|p| p.items.clone())
        .unwrap_or_default();
    let next = store
        .read()
        .page
        .as_ref()
        .and_then(|p| p.next_cursor.clone());
    let account_tools: Vec<(String, String)> = rows
        .iter()
        .map(|t| (t.id.clone(), t.name.clone()))
        .collect();
    let account_class = format!(
        "desktop-workspace {} {} {}",
        if account_preferences.read()["theme"] == "dark" {
            "account-dark"
        } else {
            ""
        },
        if account_preferences.read()["density"] == "pro" {
            "account-compact"
        } else {
            ""
        },
        if account_preferences.read()["type"] == "large" {
            "account-large"
        } else {
            ""
        }
    );
    rsx! {
       style{{include_str!("../assets/app.css")}}
       div{class:account_class,
       aside{class:"sidebar",
       div{class:"brand",span{class:"logo"}strong{"kortex"}span{class:"muted",".site"}small{if channel=="staging"{"TEST"}else{"DESKTOP"}}}
       nav{aria_label:"Navigation principale",
       for (s,label) in [(Section::Store,"Store"),(Section::Library,"Bibliothèque"),(Section::Updates,"Mises à jour"),(Section::Account,"Compte"),(Section::Ai,"Assistant"),(Section::Workflows,"Parcours")]{
       button{class:if *section.read()==s{"nav-item selected"}else{"nav-item"},aria_current:if *section.read()==s{"page"}else{"false"},disabled:*busy.read(),onclick:move |_|{section.set(s);detail.set(None);active.set(None);},"{label}"}
       }}
       div{class:"sidebar-bottom",div{class:"connection",span{class:if offline{"dot offline"}else{"dot"}}strong{if offline{"Hors connexion"}else{"Application locale"}}p{"Vos fichiers restent sur cet appareil."}button{disabled:*busy.read()||active.read().is_some(),onclick:move |_|{let next=!*offline_mode.read();offline_mode.set(next);},if offline{"Revenir en ligne"}else{"Utiliser hors connexion"}}}p{class:"muted","Kortex Desktop · 1.0"}}
       }
       main{class:"main-panel",
       header{span{class:"muted","Espace de travail"}span{"/"}strong{"{header}"}span{class:"local-badge","Windows"}}
       div{class:"content",
       if offline{p{class:"banner",role:"status","Hors connexion · vos outils installés restent disponibles."}}
       if !notice.read().is_empty(){p{class:"message",role:"status","{notice}"}}
       if *busy.read() && !progress.read().phase.is_empty(){p{role:"status","{progress.read().phase}"}progress{value:progress.read().downloaded,max:progress.read().total.unwrap_or(25*1024*1024)}{let cancel=cancel.clone();rsx!{button{onclick:move |_|cancel.store(true,std::sync::atomic::Ordering::SeqCst),"Annuler le téléchargement"}}}}
       if let Some(plan)=active.read().clone(){
       {let tool=plan.installed.clone();let cfg=config.clone().ok();let id=tool.id.clone();let version=tool.version.clone();rsx!{
       div{class:"section-title",p{class:"muted","{tool.name} · {tool.version}"}button{onclick:move |_|active.set(None),"Retour à la bibliothèque"}}
       kortex_runtime_desktop::app::ToolSession{archive:Some(tool.archive.to_string_lossy().into_owned()),trust_bundle:Some(tool.trust.to_string_lossy().into_owned()),product:true,trust_seal:Some(tool.trust_seal.clone()),session_lease:Some(plan.lease.clone()),data_root:cfg.as_ref().map(|c|c.data_dir.join("tools").to_string_lossy().into_owned()),on_started:move|result|{if let Some(c)=cfg.clone(){manager::started(c,id.clone(),version.clone(),result,active,library,notice)}}}
       }}
       }else{
       match *section.read(){
       Section::Store=>rsx!{
       h1{"Les bons outils, au même endroit."}p{class:"subtitle","Explorez le catalogue Kortex et trouvez ce qu’il vous faut."}
       form{class:"search",onsubmit:move|e|{e.prevent_default();if let Ok(api)=search_api.clone(){load(api,store,busy,status,epoch,Query{q:query.read().clone(),category:category.read().clone(),sort:sort.read().clone()},None,true)}},
       input{r#type:"search",aria_label:"Rechercher un outil",placeholder:"Rechercher un outil…",value:"{query}",maxlength:200,oninput:move|e|query.set(e.value())}
       button{r#type:"submit",disabled:*busy.read(),"Rechercher"}
       }
       div{class:"filters",
       label{r#for:"category","Catégorie"}select{id:"category",value:"{category}",disabled:*busy.read(),onchange:move|e|{category.set(e.value());if let Ok(api)=category_api.clone(){load(api,store,busy,status,epoch,Query{q:query.read().clone(),category:e.value(),sort:sort.read().clone()},None,true)}},
       option{value:"","Toutes les catégories"}
       for value in categories{option{value:value.clone(),"{category_label(&value)}"}}
       }
       label{r#for:"sort","Trier"}select{id:"sort",value:"{sort}",disabled:*busy.read(),onchange:move|e|{sort.set(e.value());if let Ok(api)=sort_api.clone(){load(api,store,busy,status,epoch,Query{q:query.read().clone(),category:category.read().clone(),sort:e.value()},None,true)}},option{value:"recent","Plus récents"}option{value:"name","Nom"}}
       }
       if *busy.read(){p{role:"status","Chargement…"}}
       if !status.read().is_empty(){p{role:"status",class:"message","{status}"}}
       if !*busy.read()&&rows.is_empty()&&status.read().is_empty(){p{class:"empty","Aucun outil ne correspond à votre recherche."}}
       div{class:"tool-grid",
       for tool in rows{
       {let api=api.clone();let id=tool.id.clone();rsx!{
       button{class:"tool-card",disabled:*busy.read(),onclick:move |_|{if let Ok(api)=api.clone(){details(api,id.clone(),detail,busy,status)}},
       span{class:"glyph","{tool.name.chars().take(2).collect::<String>().to_uppercase()}"}small{class:"category","{category_label(&tool.category)}"}h2{"{tool.name}"}p{"{tool.summary}"}span{class:"version","Version {tool.version}"}
       }}}
       }}
       if let Some(cursor)=next{
       {let api=api.clone();rsx!{button{class:"next",disabled:*busy.read(),onclick:move |_|{if let Ok(api)=api.clone(){let q=store.read().query.clone();load(api,store,busy,status,epoch,q,Some(cursor.clone()),false)}},"Page suivante"}}}
       }
       },
       Section::Library=>rsx!{
       h1{"Votre bibliothèque"}p{class:"subtitle","Vos outils installés, disponibles même hors connexion."}
       if library.read().rejected>0{p{role:"status",class:"banner","Certains fichiers locaux n’ont pas pu être vérifiés et restent désactivés."}}
       if installed.is_empty()&&library.read().unavailable.is_empty(){div{class:"empty",h2{"Votre bibliothèque est vide"}p{"Explorez le Store pour découvrir les outils disponibles."}}}
       div{class:"tool-grid",for tool in installed.clone(){
       {let item=tool.clone();let cfg=config.clone();let api=api.clone();let remove=tool.clone();let rollback=tool.clone();rsx!{article{class:"tool-card",span{class:"glyph","{tool.name.chars().take(2).collect::<String>().to_uppercase()}"}h2{"{tool.name}"}p{"Version {tool.version}"}details{summary{"Versions conservées"}p{{tool.versions.join(", ")}}}
       button{disabled:*busy.read(),onclick:move |_|{if let(Ok(c),Ok(a))=(cfg.clone(),api.clone()){manager::open(c,a,item.id.clone(),active,busy,notice,library)}},"Ouvrir"}
       button{disabled:*busy.read(),onclick:move |_|confirmation.set(Some(Choice::new(Operation::Uninstall{id:remove.id.clone(),name:remove.name.clone()}))),"Désinstaller"}
       if tool.previous.is_some(){button{disabled:*busy.read(),onclick:move |_|confirmation.set(Some(Choice::new(Operation::Rollback(rollback.clone())))),"Version précédente"}}}}}
       }}
       },
       Section::Updates=>rsx!{h1{"Mises à jour"}
    if let Ok(c)=&config{crate::runtime_update_ui::RuntimeUpdate{config:c.clone()}}
       {let cfg=config.clone();let api=api.clone();rsx!{button{disabled:*busy.read(),onclick:move |_|{if let(Ok(c),Ok(a))=(cfg.clone(),api.clone()){manager::check(c,a,busy,available,checked,notice)}},"Rechercher les mises à jour"}}}
       for update in available.read().clone(){{let candidate=update.clone();rsx!{article{class:"tool-card",h2{"{update.detail.tool.name}"}p{"{update.installed.version} → {update.detail.tool.version}"}p{"{update.detail.manifest.description}"}button{disabled:*busy.read(),onclick:move |_|confirmation.set(Some(Choice::new(Operation::Update(candidate.clone())))),"Mettre à jour"}}}}}
       },
       Section::Ai=>rsx!{if let Ok(c)=&config{crate::ai_view::AiView{config:c.clone()}}},
       Section::Workflows=>rsx!{if let Ok(c)=&config{crate::workflow_view::WorkflowView{config:c.clone()}}},
       Section::Account=>rsx!{if let Ok(c)=&config{crate::account_ui::AccountPanel{config:c.clone(),tools:account_tools,on_preferences:move|value|account_preferences.set(value)}}}
       }
       }
       for id in library.read().unavailable.clone(){if *section.read()==Section::Library{{let id2=id.clone();rsx!{div{class:"empty",p{"Outil indisponible : {id}"}button{disabled:*busy.read(),onclick:move |_|confirmation.set(Some(Choice::new(Operation::Uninstall{id:id2.clone(),name:id2.clone()}))),"Désinstaller"}}}}}}
       if let Some(d)=detail.read().clone(){
       section{role:"dialog",aria_label:"Fiche de l’outil",class:"detail",h2{"{d.tool.name}"}p{"{d.manifest.description}"}p{class:"muted","Version {d.tool.version}"}h3{"Permissions"}
       if d.manifest.permissions.is_empty(){p{"Aucune permission sensible."}}else{ul{for permission in d.manifest.permissions.clone(){li{"{permission}"}}}}
       h3{"Ce que cet outil peut faire"}ul{for action in d.contract.actions.clone(){li{"{action.description}"}}}
       {let candidate=d.clone();let already=installed.iter().any(|i|i.id==d.tool.id);rsx!{button{disabled:*busy.read()||already,onclick:move |_|{detail.set(None);confirmation.set(Some(Choice::new(Operation::Install(candidate.clone()))));},if already{"Déjà installé"}else{"Installer"}}}}
       button{onclick:move |_|detail.set(None),"Fermer la fiche"}
       }
       }
       if let Some(choice)=confirmation.read().clone(){
       {let cfg=config.clone();let api=api.clone();let cancel=cancel.clone();let monitor=monitor.clone();rsx!{manager::Confirm{choice:choice.clone(),on_cancel:move |_|confirmation.set(None),on_confirm:move|purge|{confirmation.set(None);if let(Ok(c),Ok(a))=(cfg.clone(),api.clone()){manager::execute(c,a,choice.clone(),purge,cancel.clone(),monitor.clone(),busy,library,available,notice)}}}}}
       }
       }
       }
       }
       }
}
