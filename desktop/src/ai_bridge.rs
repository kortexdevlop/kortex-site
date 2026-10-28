use crate::{api::Api, config::Config, launcher, library};
use serde_json::Value;
struct Clock;
impl kortex_core::dispatcher::Clock for Clock {
    fn now_ms(&self) -> u64 {
        kortex_runtime_native::permissions::now_ms()
    }
    async fn wait_until(&self, deadline: u64) {
        tokio::time::sleep(std::time::Duration::from_millis(
            deadline.saturating_sub(kortex_runtime_native::permissions::now_ms()),
        ))
        .await
    }
}
/// The UI must display and approve this immutable single step first. No capability grants are created here.
pub fn execute(c: &Config, plan: Value) -> Result<Value, String> {
    if !kortex_core::ai_bridge::bounded_plan(&plan)
        || plan["steps"].as_array().is_none_or(|s| s.len() != 1)
    {
        return Err("Proposition invalide.".into());
    }
    let tool_id = plan["steps"][0]["tool_id"]
        .as_str()
        .ok_or("Outil inconnu.")?;
    let api = Api::new(c).map_err(|e| e.to_string())?;
    let launch = launcher::prepare(c, &api, tool_id)?;
    let installed = &launch.installed;
    if !c.offline {
        let current = api.detail_fresh(tool_id).map_err(|e| e.to_string())?;
        if current.tool.version != installed.version
            || current.descriptor.sha256 != installed.sha256
        {
            return Err("La version publiée a changé. Actualisez cet outil depuis le Store avant de confirmer.".into());
        }
    }
    let tool = kortex_runtime_desktop::app::open_verified_with_pin(
        installed.archive.to_str().ok_or("Paquet invalide.")?,
        installed.trust.to_str().ok_or("Clés invalides.")?,
        Some(&installed.trust_seal),
    )?;
    let action = tool
        .contract()
        .actions
        .iter()
        .find(|a| Some(a.id.as_str()) == plan["steps"][0]["action_id"].as_str())
        .ok_or("Action inconnue.")?;
    if !action.permissions.is_empty() || action.side_effect != "none" {
        return Err("Cette action demande un accès sensible. Ouvrez l’outil pour accorder ses autorisations ; aucune autorisation n’est créée par l’IA.".into());
    }
    let inbox = kortex_runtime_native::permissions::CapabilityInbox::default();
    let engine = kortex_runtime_native::HostedNativeEngine::new(inbox.clone());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|_| "Moteur indisponible.")?;
    runtime.block_on(async{let mut host=kortex_core::Runtime::new(engine,Clock);let id=host.load(tool).await.map_err(|_|"Paquet refusé.")?;
 let result=async{host.initialize(id).await.map_err(|_|"Initialisation refusée.")?;if inbox.pop().is_some(){return Err("L’outil demande une permission au démarrage. Ouvrez-le depuis la bibliothèque.".into())}let mut review=host.review_ai_plan(&plan).map_err(|_|"Proposition refusée par le moteur local.")?;let result=host.execute_ai_step(&mut review,0,true).await.map_err(|_|"L’action locale a échoué.")?;if inbox.pop().is_some(){return Err("Accès sensible refusé : utilisez l’interface de l’outil.".into())}Ok(result)}.await;let stopped=host.shutdown(id).await;if stopped.is_err(){return Err("Arrêt du moteur incomplet.".into())}result})
}
pub fn installed_ids(c: &Config) -> Vec<String> {
    library::scan(c)
        .tools
        .into_iter()
        .map(|i| i.id)
        .take(10)
        .collect()
}
pub fn execute_reviewed(
    c: &Config,
    plan: Value,
    owner: Option<&str>,
    created: u64,
) -> Result<Value, String> {
    let now = kortex_runtime_native::permissions::now_ms();
    if now < created || now.saturating_sub(created) >= 300000 {
        return Err("Proposition expirée. Recommencez la demande.".into());
    }
    if let Some(expected) = owner {
        if c.offline {
            return Err("Reconnectez-vous pour vérifier cette proposition du compte.".into());
        }
        let session = crate::secure_store::load(c)?.ok_or("Compte déconnecté.")?;
        if session.user_id != expected
            || session.expires_at <= now
            || !crate::account::view(c)?.connected
        {
            return Err("Le compte a changé ou a été déconnecté.".into());
        }
    }
    execute(c, plan)
}
