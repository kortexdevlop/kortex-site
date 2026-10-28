use crate::{
    api::Api,
    config::Config,
    library::{self, Installed},
    paths,
};
use std::sync::Arc;
#[derive(Clone)]
pub struct LaunchPlan {
    pub installed: Installed,
    pub lease: kortex_runtime_desktop::app::SessionLease,
}
pub fn prepare(c: &Config, api: &Api, id: &str) -> Result<LaunchPlan, String> {
    let lock = library::state_lock(c)?;
    let installed = match library::current(c, id) {
        Ok(i) => i,
        Err(error) => {
            let pending = library::load(c)?
                .tools
                .get(id)
                .is_some_and(|e| e.pending.is_some());
            drop(lock);
            if pending {
                crate::updates::rollback(c, id)?;
                return Err(format!(
                    "La nouvelle version ne peut pas démarrer ; la version précédente a été restaurée : {error}"
                ));
            }
            return Err(error);
        }
    };
    let lease = library::lease(c, id, true)?;
    drop(lock);
    if !c.offline {
        let d = kortex_verify::Descriptor::parse(&paths::read(
            &installed.archive.with_extension("tool.json"),
            1048576,
        )?)
        .map_err(|_| "Descripteur invalide.")?;
        if !api.availability(&d).map_err(|e| e.to_string())? {
            drop(lease);
            library::mark_blocked(c, id, &installed.version)?;
            if library::load(c)?
                .tools
                .get(id)
                .is_some_and(|e| e.pending.is_some())
            {
                crate::updates::rollback(c, id)?;
            }
            return Err("Cette version a été retirée ou révoquée ; son lancement est bloqué, même hors connexion.".into());
        }
        api.detail_fresh(id).map_err(|e| e.to_string())?;
    }
    Ok(LaunchPlan {
        installed,
        lease: kortex_runtime_desktop::app::SessionLease(Arc::new(lease)),
    })
}
struct Clock;
impl kortex_core::dispatcher::Clock for Clock {
    fn now_ms(&self) -> u64 {
        kortex_runtime_native::permissions::now_ms()
    }
    async fn wait_until(&self, deadline: u64) {
        tokio::time::sleep(std::time::Duration::from_millis(
            deadline.saturating_sub(self.now_ms()),
        ))
        .await
    }
}
pub fn probe(i: &Installed) -> Result<(), String> {
    let tool = kortex_runtime_desktop::app::open_verified_with_pin(
        i.archive.to_str().ok_or("Chemin invalide.")?,
        i.trust.to_str().ok_or("Chemin invalide.")?,
        Some(&i.trust_seal),
    )?;
    let inbox = kortex_runtime_native::permissions::CapabilityInbox::default();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|_| "Moteur indisponible.")?;
    runtime.block_on(async{let mut host=kortex_core::Runtime::new(kortex_runtime_native::HostedNativeEngine::new(inbox.clone()),Clock);let id=host.load(tool).await.map_err(|e|format!("Démarrage refusé : {e:?}"))?;
 let init=host.initialize(id).await.map(|_|());let pending=inbox.pop().is_some();let stopped=host.shutdown(id).await;
 init.map_err(|e|format!("Initialisation refusée : {e:?}"))?;stopped.map_err(|_|"Arrêt de validation impossible.")?;
 if pending{return Err("Cette version demande une migration ou un accès sensible au démarrage ; l’ancienne version est conservée.".into())}Ok(())})
}
