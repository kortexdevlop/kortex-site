use crate::{ActionSpec, Compiled, Error};
use kortex_capabilities::{
    Capability, Resource,
    policy::{Decision, Policy},
};
use std::collections::BTreeSet;
#[derive(Clone)]
pub struct Scope {
    pub capability: Capability,
    pub resource: Resource,
}
#[derive(Debug, Clone)]
pub struct Preview {
    pub node_id: String,
    pub action_ref: String,
    pub permissions: Vec<String>,
    pub side_effect: String,
    pub confirmation: bool,
}
pub fn preview(plan: &Compiled) -> Vec<Preview> {
    plan.workflow
        .nodes
        .iter()
        .zip(&plan.specs)
        .map(|(n, s)| Preview {
            node_id: n.id.clone(),
            action_ref: n.action_ref.clone(),
            permissions: s.action.permissions.clone(),
            side_effect: s.action.side_effect.clone(),
            confirmation: sensitive(n.requires_confirmation, s),
        })
        .collect()
}
pub fn aggregated_permissions(plan: &Compiled) -> Vec<String> {
    plan.specs
        .iter()
        .flat_map(|s| s.action.permissions.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
pub fn sensitive(explicit: bool, spec: &ActionSpec) -> bool {
    explicit || !spec.action.permissions.is_empty() || spec.action.side_effect != "none"
}
/// Workflow consent never calls Policy::grant; scope/gesture grants remain with the normal runtime host UI.
pub struct Consent {
    pub(crate) run_id: u64,
    pub(crate) retries: u8,
    fingerprint: String,
    expires: u64,
    pure: bool,
    approved: BTreeSet<String>,
    denied: BTreeSet<String>,
    sealed: bool,
}
impl Consent {
    pub(crate) fn deadline(&self) -> u64 {
        self.expires
    }
    pub(crate) fn new(
        run_id: u64,
        retries: u8,
        fingerprint: String,
        expires: u64,
        allow_pure: bool,
    ) -> Self {
        Self {
            run_id,
            retries,
            fingerprint,
            expires,
            pure: allow_pure,
            approved: BTreeSet::new(),
            denied: BTreeSet::new(),
            sealed: false,
        }
    }
    pub fn approve_node(&mut self, plan: &Compiled, node: &str) -> Result<(), Error> {
        self.approve_batch(plan, &[node.to_owned()])
    }
    pub fn approve_batch(&mut self, plan: &Compiled, nodes: &[String]) -> Result<(), Error> {
        if self.sealed
            || self.fingerprint != plan.fingerprint
            || nodes.is_empty()
            || nodes.len() > 50
        {
            return Err(Error::PermissionDenied);
        }
        for id in nodes {
            if !plan.workflow.nodes.iter().any(|n| &n.id == id) {
                return Err(Error::PermissionDenied);
            }
        }
        for id in nodes {
            self.denied.remove(id);
            self.approved.insert(id.clone());
        }
        Ok(())
    }
    pub fn deny_node(&mut self, node: &str) {
        self.approved.remove(node);
        self.denied.insert(node.into());
    }
    pub(crate) fn check(
        &self,
        run: u64,
        retries: u8,
        node: &str,
        sensitive: bool,
        now: u64,
    ) -> Result<(), Error> {
        if self.sealed || run != self.run_id || self.retries != retries {
            return Err(Error::PermissionDenied);
        }
        if now >= self.expires {
            return Err(Error::Expired);
        }
        if self.denied.contains(node) {
            return Err(Error::Cancelled);
        }
        if !self.approved.contains(node) && (sensitive || !self.pure) {
            return Err(Error::AwaitingConsent);
        }
        Ok(())
    }
    pub(crate) fn consume(&mut self, node: &str) {
        self.approved.remove(node);
    }
    pub(crate) fn seal(&mut self) {
        self.sealed = true;
        self.approved.clear();
    }
}
pub fn check_policy(
    policy: &Policy,
    spec: &ActionSpec,
    scopes: &[Scope],
    now: u64,
) -> Result<(), Error> {
    if scopes.len() != spec.action.permissions.len() {
        return Err(Error::PermissionDenied);
    }
    let mut seen = BTreeSet::new();
    for name in &spec.action.permissions {
        let cap = Capability::try_from(name.as_str()).map_err(|_| Error::PermissionDenied)?;
        if !seen.insert(cap) {
            return Err(Error::PermissionDenied);
        }
        let matching = scopes
            .iter()
            .filter(|s| s.capability == cap)
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err(Error::PermissionDenied);
        }
        match policy.consent_decision(&spec.context, cap, &matching[0].resource, now) {
            Decision::AllowOnce | Decision::AllowSession => {}
            Decision::Ask => return Err(Error::AwaitingConsent),
            Decision::Deny => return Err(Error::PermissionDenied),
        }
    }
    Ok(())
}
