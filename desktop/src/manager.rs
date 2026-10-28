use crate::{
    api::Api,
    config::Config,
    installer::{self, Monitor},
    launcher::{self, LaunchPlan},
    library::{self, Installed, Library},
    models::ToolDetail,
    updates::{self, Update},
};
use dioxus::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
#[derive(Clone, Debug)]
pub enum Operation {
    Install(ToolDetail),
    Update(Update),
    Uninstall { id: String, name: String },
    Rollback(Installed),
}
#[derive(Clone, Debug)]
pub struct Choice(pub Arc<Operation>);
impl PartialEq for Choice {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Choice {
    pub fn new(o: Operation) -> Self {
        Self(Arc::new(o))
    }
}
pub fn execute(
    c: Config,
    api: Api,
    choice: Choice,
    purge: bool,
    cancel: Arc<AtomicBool>,
    monitor: Monitor,
    mut busy: Signal<bool>,
    mut library: Signal<Library>,
    mut available: Signal<Vec<Update>>,
    mut notice: Signal<String>,
) {
    if *busy.peek() {
        return;
    }
    busy.set(true);
    cancel.store(false, Ordering::SeqCst);
    *monitor.lock().unwrap() = installer::Progress::default();
    let done = monitor.clone();
    notice.set(String::new());
    spawn(async move {
        let local = c.clone();
        let result = tokio::task::spawn_blocking(move || match choice.0.as_ref() {
            Operation::Install(d) => installer::install(
                &c,
                &api,
                d,
                &d.manifest.permissions,
                &cancel,
                &monitor,
                true,
            )
            .map(|_| "Outil installé.".to_string()),
            Operation::Update(u) => updates::apply(
                &c,
                &api,
                u,
                &u.detail.manifest.permissions,
                &cancel,
                &monitor,
            )
            .map(|_| "Mise à jour installée. Ouvrez l’outil pour valider son démarrage.".into()),
            Operation::Uninstall { id, .. } => library::uninstall(&c, id, purge).map(|_| {
                if purge {
                    "Outil désinstallé ; les données de ce profil ont été effacées.".into()
                } else {
                    "Outil désinstallé ; ses données ont été conservées.".into()
                }
            }),
            Operation::Rollback(i) => {
                updates::rollback(&c, &i.id).map(|_| "Version précédente restaurée.".into())
            }
        })
        .await;
        done.lock().unwrap().phase.clear();
        notice.set(match result {
            Ok(Ok(message)) => message,
            Ok(Err(e)) => e,
            Err(_) => "Opération interrompue.".into(),
        });
        if let Ok(v) = tokio::task::spawn_blocking(move || library::scan(&local)).await {
            library.set(v)
        }
        available.set(vec![]);
        busy.set(false);
    });
}
pub fn open(
    c: Config,
    api: Api,
    id: String,
    mut active: Signal<Option<LaunchPlan>>,
    mut busy: Signal<bool>,
    mut notice: Signal<String>,
    mut library: Signal<Library>,
) {
    if *busy.peek() {
        return;
    }
    busy.set(true);
    notice.set(String::new());
    let local = c.clone();
    spawn(async move {
        match tokio::task::spawn_blocking(move || launcher::prepare(&c, &api, &id)).await {
            Ok(Ok(plan)) => active.set(Some(plan)),
            Ok(Err(e)) => notice.set(e),
            Err(_) => notice.set("Ouverture impossible.".into()),
        }
        if let Ok(v) = tokio::task::spawn_blocking(move || library::scan(&local)).await {
            library.set(v)
        }
        busy.set(false);
    });
}
pub fn started(
    c: Config,
    id: String,
    version: String,
    result: Result<(), String>,
    mut active: Signal<Option<LaunchPlan>>,
    mut library: Signal<Library>,
    mut notice: Signal<String>,
) {
    if result.is_err() {
        active.set(None);
    }
    spawn(async move {
        if result.is_err() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let local = c.clone();
        let outcome=tokio::task::spawn_blocking(move||{
  match result{Ok(())=>updates::confirm_launch(&c,&id,&version).map(|_|String::new()),Err(e)=>{
   if library::load(&c)?.tools.get(&id).is_some_and(|r|r.pending.is_some()){updates::rollback(&c,&id).map(|_|"Le démarrage de la mise à jour a échoué. La version précédente et les données ont été restaurées.".into())}else{Err(e)}
  }}
 }).await;
        match outcome {
            Ok(Ok(message)) if !message.is_empty() => notice.set(message),
            Ok(Err(e)) => notice.set(e),
            Err(_) => notice.set("Enregistrement du démarrage impossible.".into()),
            _ => {}
        }
        if let Ok(v) = tokio::task::spawn_blocking(move || library::scan(&local)).await {
            library.set(v)
        }
    });
}
pub fn check(
    c: Config,
    api: Api,
    mut busy: Signal<bool>,
    mut available: Signal<Vec<Update>>,
    mut checked: Signal<bool>,
    mut notice: Signal<String>,
) {
    if *busy.peek() {
        return;
    }
    busy.set(true);
    notice.set(String::new());
    spawn(async move {
        match tokio::task::spawn_blocking(move || updates::check(&c, &api)).await {
            Ok(Ok(v)) => {
                if v.is_empty() {
                    notice.set("Aucune mise à jour disponible.".into())
                }
                available.set(v);
                checked.set(true)
            }
            Ok(Err(e)) => notice.set(e),
            Err(_) => notice.set("Vérification impossible.".into()),
        }
        busy.set(false);
    });
}
fn permission(p: &str) -> &str {
    match p {
        "file.pick" => "Choisir un fichier",
        "file.read" => "Lire le fichier choisi",
        "file.save" => "Enregistrer un fichier",
        "storage.read" => "Lire les préférences de cet outil",
        "storage.write" => "Mémoriser les préférences de cet outil",
        "network.fetch" => "Accéder aux adresses réseau autorisées",
        "clipboard.write" => "Écrire dans le presse-papiers",
        _ => "Permission supplémentaire",
    }
}
#[component]
pub fn Confirm(
    choice: Choice,
    on_confirm: EventHandler<bool>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut purge = use_signal(|| false);
    let(title,name,permissions,description)=match choice.0.as_ref(){
 Operation::Install(d)=>("Installer cet outil",d.tool.name.clone(),d.manifest.permissions.clone(),d.manifest.description.clone()),
 Operation::Update(u)=>("Confirmer la mise à jour",u.detail.tool.name.clone(),u.extra_permissions.clone(),u.detail.manifest.description.clone()),
 Operation::Uninstall{name,..}=>("Confirmer la désinstallation",name.clone(),vec![],"Choisissez de conserver ou d’effacer les données de cet outil pour ce profil.".into()),
 Operation::Rollback(i)=>("Restaurer la version précédente",i.name.clone(),vec![],"La version précédente sera réactivée. Vos données de ce profil seront conservées ou restaurées si le premier démarrage n’a pas été validé.".into())
 };
    let uninstall = matches!(choice.0.as_ref(), Operation::Uninstall { .. });
    let updating = matches!(choice.0.as_ref(), Operation::Update(_));
    rsx! {section{class:"detail",role:"dialog",aria_label:"Confirmation",h2{"{title}"}h3{"{name}"}p{"{description}"}
    if uninstall{label{input{r#type:"checkbox",checked:*purge.read(),onchange:move|e|purge.set(e.checked())}"Effacer aussi les données de cet outil pour ce profil"}}else if !permissions.is_empty(){h3{if updating{"Permissions supplémentaires"}else{"Permissions de l’outil"}}ul{for p in permissions{li{"{permission(&p)}"}}}p{"L’installation n’accorde aucun accès automatiquement : chaque demande sensible sera confirmée pendant l’utilisation."}}else{p{if updating{"Aucune permission supplémentaire."}else{"Aucune permission sensible."}}}
    button{onclick:move |_|on_cancel.call(()),"Annuler"}button{onclick:move |_|on_confirm.call(*purge.read()),"{title}"}
    }}
}
