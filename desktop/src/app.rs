use crate::{
    api::{Api, Query},
    config::Config,
    library::{self, Installed, Library},
    models::ToolDetail,
    store::Store,
};
use dioxus::prelude::*;
#[derive(Clone, Copy, PartialEq)]
enum Section {
    Store,
    Library,
    Updates,
    Account,
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
            let d = api.detail(&id)?;
            let actions = api.actions(&id, &d.tool.version)?;
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
    let mut section = use_signal(|| Section::Store);
    let mut store = use_signal(Store::default);
    let mut busy = use_signal(|| false);
    let mut status = use_signal(String::new);
    let epoch = use_signal(|| 0u64);
    let mut query = use_signal(String::new);
    let mut category = use_signal(String::new);
    let mut sort = use_signal(|| "recent".to_string());
    let mut detail = use_signal(|| None::<ToolDetail>);
    let mut active = use_signal(|| None::<Installed>);
    let mut library = use_signal(Library::default);
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
    rsx! {
    style{{include_str!("../assets/app.css")}}
    div{class:"desktop-workspace",
    aside{class:"sidebar",
    div{class:"brand",span{class:"logo"}strong{"kortex"}span{class:"muted",".site"}small{if channel=="staging"{"TEST"}else{"DESKTOP"}}}
    nav{aria_label:"Navigation principale",
    for (s,label) in [(Section::Store,"Store"),(Section::Library,"Bibliothèque"),(Section::Updates,"Mises à jour"),(Section::Account,"Compte")]{
    button{class:if *section.read()==s{"nav-item selected"}else{"nav-item"},aria_current:if *section.read()==s{"page"}else{"false"},onclick:move |_|{section.set(s);detail.set(None);active.set(None);},"{label}"}
    }}
    div{class:"sidebar-bottom",div{class:"connection",span{class:if offline{"dot offline"}else{"dot"}}strong{if offline{"Hors connexion"}else{"Application locale"}}p{"Vos fichiers restent sur cet appareil."}}p{class:"muted","Kortex Desktop · 1.0"}}
    }
    main{class:"main-panel",
    header{span{class:"muted","Espace de travail"}span{"/"}strong{"{header}"}span{class:"local-badge","Windows"}}
    div{class:"content",
    if offline{p{class:"banner",role:"status","Hors connexion · vos outils installés restent disponibles."}}
    if let Some(tool)=active.read().clone(){
    div{class:"section-title",p{class:"muted","{tool.name} · {tool.version}"}button{onclick:move |_|active.set(None),"Retour à la bibliothèque"}}
    kortex_runtime_desktop::app::ToolSession{archive:Some(tool.archive.to_string_lossy().into_owned()),trust_bundle:Some(tool.trust.to_string_lossy().into_owned()),product:true}
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
    if installed.is_empty(){div{class:"empty",h2{"Votre bibliothèque est vide"}p{"Explorez le Store pour découvrir les outils disponibles."}}}
    div{class:"tool-grid",for tool in installed{
    {let item=tool.clone();rsx!{article{class:"tool-card",span{class:"glyph","{tool.name.chars().take(2).collect::<String>().to_uppercase()}"}h2{"{tool.name}"}p{"Version {tool.version}"}button{onclick:move |_|active.set(Some(item.clone())),"Ouvrir"}}}}
    }}
    },
    Section::Updates=>rsx!{h1{"Mises à jour"}div{class:"empty",p{"Cet espace regroupera les mises à jour de vos outils."}}},
    Section::Account=>rsx!{h1{"Mon espace"}div{class:"empty",h2{"Vous utilisez Kortex sans compte"}p{"Votre bibliothèque locale reste accessible sans connexion à un compte."}}}
    }
    }
    if let Some(d)=detail.read().clone(){
    section{role:"dialog",aria_label:"Fiche de l’outil",class:"detail",h2{"{d.tool.name}"}p{"{d.manifest.description}"}p{class:"muted","Version {d.tool.version}"}h3{"Permissions"}
    if d.manifest.permissions.is_empty(){p{"Aucune permission sensible."}}else{ul{for permission in d.manifest.permissions{li{"{permission}"}}}}
    h3{"Ce que cet outil peut faire"}ul{for action in d.contract.actions{li{"{action.description}"}}}
    button{onclick:move |_|detail.set(None),"Fermer la fiche"}
    }
    }
    }
    }
    }
    }
}
