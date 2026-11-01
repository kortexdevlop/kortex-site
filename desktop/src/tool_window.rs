use crate::{
    api::Api,
    config::Config,
    launcher::{self, LaunchPlan},
    library::{self, Library},
    manager, paths,
};
use dioxus::prelude::*;
use std::path::{Path, PathBuf};
const FLAG: &str = "--tool";
pub fn requested(mut args: impl Iterator<Item = String>) -> Option<String> {
    args.next();
    let (flag, id, extra) = (args.next()?, args.next()?, args.next());
    (flag == FLAG && extra.is_none() && crate::api::slug(&id)).then_some(id)
}
pub fn requested_now() -> Option<String> {
    requested(std::env::args_os().map(|a| a.to_string_lossy().into_owned()))
}
fn handle_path(c: &Config, id: &str) -> PathBuf {
    c.data_dir.join("sessions").join(format!("{id}.hwnd"))
}
/// Publishes the window showing a tool so a later launch can bring it forward.
pub fn announce(c: &Config, id: &str) {
    #[cfg(windows)]
    {
        use dioxus::desktop::tao::platform::windows::WindowExtWindows;
        let handle = dioxus::desktop::window().hwnd();
        let _ = paths::atomic(&handle_path(c, id), handle.to_string().as_bytes());
    }
    #[cfg(not(windows))]
    let _ = (c, id);
}
/// True when the tool already has a session; its window is then brought forward.
pub fn focus_open(c: &Config, id: &str) -> bool {
    if library::session(c, id).is_ok() {
        return false;
    }
    #[cfg(windows)]
    if let Some(handle) = paths::read(&handle_path(c, id), 32)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .and_then(|s| s.parse::<isize>().ok())
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            IsIconic, IsWindow, SW_RESTORE, SetForegroundWindow, ShowWindow,
        };
        let window = handle as *mut core::ffi::c_void;
        // SAFETY: these calls take a plain handle and reject one that is no longer valid.
        unsafe {
            if IsWindow(window) != 0 {
                if IsIconic(window) != 0 {
                    ShowWindow(window, SW_RESTORE);
                }
                SetForegroundWindow(window);
            }
        }
    }
    true
}
pub fn link_path(folder: &Path, name: &str, id: &str) -> PathBuf {
    let clean: String = name
        .chars()
        .filter(|c| !c.is_control() && !r#"<>:"/\|?*"#.contains(*c))
        .collect();
    let clean = clean.trim().trim_end_matches(['.', ' ']);
    folder.join(format!("{}.lnk", if clean.is_empty() { id } else { clean }))
}
/// Returns false when a file of that name exists: it may not be ours, so it is left alone.
pub fn create_shortcut(folder: &Path, exe: &Path, id: &str, name: &str) -> Result<bool, String> {
    if !crate::api::slug(id) {
        return Err("Identifiant invalide.".into());
    }
    let link = link_path(folder, name, id);
    if link.exists() {
        return Ok(false);
    }
    write_link(&link, exe, &format!("{FLAG} {id}"), name)?;
    Ok(true)
}
#[cfg(windows)]
fn write_link(link: &Path, exe: &Path, arguments: &str, description: &str) -> Result<(), String> {
    use windows::{
        Win32::{
            System::Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoUninitialize, IPersistFile,
            },
            UI::Shell::{IShellLinkW, ShellLink},
        },
        core::{HSTRING, Interface},
    };
    let failed = || "Création du raccourci impossible.".to_string();
    // SAFETY: COM stays initialized on this thread for every call below.
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|_| failed())?;
        let result = (|| -> windows::core::Result<()> {
            let shell: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
            shell.SetPath(&HSTRING::from(exe))?;
            shell.SetArguments(&HSTRING::from(arguments))?;
            shell.SetDescription(&HSTRING::from(description))?;
            if let Some(folder) = exe.parent() {
                shell.SetWorkingDirectory(&HSTRING::from(folder))?;
            }
            shell
                .cast::<IPersistFile>()?
                .Save(&HSTRING::from(link), true)
        })();
        CoUninitialize();
        result.map_err(|_| failed())
    }
}
#[cfg(not(windows))]
fn write_link(_: &Path, _: &Path, _: &str, _: &str) -> Result<(), String> {
    Err("Raccourcis disponibles sur Windows uniquement.".into())
}
pub(crate) fn open_separately(id: &str, mut notice: Signal<String>) {
    let started = std::env::current_exe()
        .and_then(|exe| std::process::Command::new(exe).args([FLAG, id]).spawn());
    if started.is_err() {
        notice.set("Ouverture de la fenêtre impossible.".into());
    }
}
pub(crate) fn add_shortcut(id: String, name: String, mut notice: Signal<String>) {
    spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            let exe = std::env::current_exe().map_err(|_| "Application introuvable.")?;
            let desktop = dirs::desktop_dir().ok_or("Bureau introuvable.")?;
            create_shortcut(&desktop, &exe, &id, &name)
        })
        .await;
        notice.set(match result {
            Ok(Ok(true)) => "Raccourci ajouté au bureau.".into(),
            Ok(Ok(false)) => "Un raccourci à ce nom existe déjà sur le bureau.".into(),
            Ok(Err(e)) => e,
            Err(_) => "Création du raccourci impossible.".into(),
        });
    });
}
fn launch(
    config: Result<Config, String>,
    id: String,
    offline: bool,
    mut active: Signal<Option<LaunchPlan>>,
    mut notice: Signal<String>,
    mut can_go_offline: Signal<bool>,
) {
    notice.set(String::new());
    can_go_offline.set(false);
    spawn(async move {
        let retry = config.clone();
        let session = config.clone();
        let wanted = id.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut c = config?;
            c.offline |= offline;
            let api = Api::new(&c)
                .map_err(|e| e.to_string())?
                .offline_mode(c.offline);
            launcher::prepare_session(&c, &api, &wanted)
        })
        .await
        .unwrap_or_else(|_| Err("Ouverture impossible.".into()));
        match result {
            Ok(plan) => {
                dioxus::desktop::window().set_title(&format!("{} — Kortex", plan.installed.name));
                if let Ok(c) = &session {
                    announce(c, &id);
                }
                active.set(Some(plan));
            }
            Err(error) => {
                notice.set(error);
                can_go_offline.set(
                    !offline && {
                        tokio::task::spawn_blocking(move || {
                            retry.is_ok_and(|c| {
                                !c.offline
                                    && library::load(&c).is_ok_and(|i| i.tools.contains_key(&id))
                            })
                        })
                        .await
                        .unwrap_or(false)
                    },
                );
            }
        }
    });
}
#[component]
pub fn ToolWindow() -> Element {
    let id = use_hook(|| requested_now().unwrap_or_default());
    let config = use_hook(Config::load);
    let mut offline = use_signal(|| false);
    let active = use_signal(|| None::<LaunchPlan>);
    let library = use_signal(Library::default);
    let notice = use_signal(String::new);
    let can_go_offline = use_signal(|| false);
    {
        let (config, id) = (config.clone(), id.clone());
        use_hook(move || launch(config, id, false, active, notice, can_go_offline));
    }
    let again = (config.clone(), id.clone());
    let without_network = (config.clone(), id.clone());
    rsx! {
       style{{include_str!("../assets/app.css")}}
       div{class:"desktop-workspace tool-window",main{class:"main-panel",div{class:"content",
       if !notice.read().is_empty(){p{class:"message",role:"status","{notice}"}}
       if let Some(plan)=active.read().clone(){
       {let tool=plan.installed.clone();let cfg=config.clone().ok();let id=tool.id.clone();let version=tool.version.clone();rsx!{
       kortex_runtime_desktop::app::ToolSession{archive:Some(tool.archive.to_string_lossy().into_owned()),trust_bundle:Some(tool.trust.to_string_lossy().into_owned()),product:true,trust_seal:Some(tool.trust_seal.clone()),session_lease:Some(plan.lease.clone()),data_root:cfg.as_ref().map(|c|c.data_dir.join("tools").to_string_lossy().into_owned()),on_started:move|result|{if let Some(c)=cfg.clone(){manager::started(c,id.clone(),version.clone(),result,active,library,notice)}}}
       }}
       }else if notice.read().is_empty(){p{role:"status","Ouverture de l’outil…"}}else{
       button{onclick:move |_|launch(again.0.clone(),again.1.clone(),*offline.read(),active,notice,can_go_offline),"Réessayer"}
       if *can_go_offline.read(){button{onclick:move |_|{offline.set(true);launch(without_network.0.clone(),without_network.1.clone(),true,active,notice,can_go_offline)},"Ouvrir hors connexion"}}
       button{onclick:move |_|{if let Ok(exe)=std::env::current_exe(){let _=std::process::Command::new(exe).spawn();}},"Ouvrir Kortex Store"}
       }
       }}}
    }
}
