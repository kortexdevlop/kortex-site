use crate::{Capability, CapabilityError, Resource};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    pub tool_id: String,
    pub version: String,
    pub instance_id: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Deny,
    Ask,
    AllowOnce,
    AllowSession,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duration {
    Once,
    Session,
}
#[derive(Debug, Clone)]
pub struct GestureToken {
    context: Context,
    expires_ms: u64,
    id: String,
}
struct Grant {
    context: Context,
    capability: Capability,
    scope: String,
    expires_ms: u64,
    duration: Duration,
}
#[derive(Default)]
pub struct Policy {
    revocation_revision: u64,
    denied: BTreeSet<(Context, Capability, String)>,
    declared: BTreeMap<Context, BTreeSet<Capability>>,
    grants: Vec<Grant>,
    prompting: BTreeSet<Context>,
    gestures: BTreeMap<String, (Context, u64)>,
}
impl Policy {
    /// Hosts use this to stop a workflow after withdrawal, without confusing a consumed one-shot grant with revocation.
    pub fn revocation_revision(&self) -> u64 {
        self.revocation_revision
    }
    pub fn declare(
        &mut self,
        context: Context,
        capabilities: impl IntoIterator<Item = Capability>,
    ) {
        self.declared
            .insert(context, capabilities.into_iter().collect());
    }
    pub fn is_declared(&self, context: &Context, capability: Capability) -> bool {
        self.declared
            .get(context)
            .is_some_and(|c| c.contains(&capability))
    }
    pub fn deny(
        &mut self,
        context: &Context,
        capability: Capability,
        resource: &Resource,
    ) -> Result<(), CapabilityError> {
        let scope = resource.scope(capability)?;
        self.grants
            .retain(|g| !(g.context == *context && g.capability == capability && g.scope == scope));
        self.denied.insert((context.clone(), capability, scope));
        self.revocation_revision = self.revocation_revision.saturating_add(1);
        Ok(())
    }
    pub fn enable_prompting(&mut self, context: &Context) {
        self.prompting.insert(context.clone());
    }
    /// Called only by the trusted host after showing scope, capability and duration to the user.
    pub fn grant(
        &mut self,
        context: &Context,
        capability: Capability,
        resource: &Resource,
        duration: Duration,
        expires_ms: u64,
    ) -> Result<(), CapabilityError> {
        if !self
            .declared
            .get(context)
            .is_some_and(|c| c.contains(&capability))
        {
            return Err(CapabilityError::Denied);
        }
        let scope = resource.scope(capability)?;
        self.denied
            .remove(&(context.clone(), capability, scope.clone()));
        self.grants.push(Grant {
            context: context.clone(),
            capability,
            scope,
            expires_ms,
            duration,
        });
        Ok(())
    }
    /// Host UI only; never accept a gesture token serialized by a guest.
    pub fn record_gesture(
        &mut self,
        context: &Context,
        now_ms: u64,
    ) -> Result<GestureToken, CapabilityError> {
        let id = crate::handles::random_id()?;
        self.gestures.retain(|_, (_, expires)| *expires > now_ms);
        self.gestures
            .insert(id.clone(), (context.clone(), now_ms.saturating_add(1000)));
        Ok(GestureToken {
            context: context.clone(),
            expires_ms: now_ms.saturating_add(1000),
            id,
        })
    }
    fn gesture_valid(&self, context: &Context, gesture: Option<&GestureToken>, now: u64) -> bool {
        gesture.is_some_and(|g| {
            g.context == *context && g.expires_ms > now && self.gestures.contains_key(&g.id)
        })
    }
    pub fn authorize(
        &self,
        context: &Context,
        capability: Capability,
        resource: &Resource,
        gesture: Option<&GestureToken>,
        now: u64,
    ) -> Decision {
        if !self
            .declared
            .get(context)
            .is_some_and(|c| c.contains(&capability))
        {
            return Decision::Deny;
        }
        if capability.needs_gesture() && !self.gesture_valid(context, gesture, now) {
            return Decision::Deny;
        }
        self.consent_decision(context, capability, resource, now)
    }
    /// Read-only host UI decision. This never authorizes an effect or creates a gesture.
    pub fn consent_decision(
        &self,
        context: &Context,
        capability: Capability,
        resource: &Resource,
        now: u64,
    ) -> Decision {
        if !self.is_declared(context, capability) {
            return Decision::Deny;
        }
        let Ok(scope) = resource.scope(capability) else {
            return Decision::Deny;
        };
        if self
            .denied
            .contains(&(context.clone(), capability, scope.clone()))
        {
            return Decision::Deny;
        }
        if let Some(grant) = self.grants.iter().find(|g| {
            g.context == *context
                && g.capability == capability
                && g.scope == scope
                && g.expires_ms > now
        }) {
            return match grant.duration {
                Duration::Once => Decision::AllowOnce,
                Duration::Session => Decision::AllowSession,
            };
        }
        if self.prompting.contains(context) {
            Decision::Ask
        } else {
            Decision::Deny
        }
    }
    pub(crate) fn permit_expiry(
        &self,
        context: &Context,
        capability: Capability,
        resource: &Resource,
        gesture: Option<&GestureToken>,
        now: u64,
    ) -> Option<u64> {
        let scope = resource.scope(capability).ok()?;
        let expiry = self
            .grants
            .iter()
            .find(|g| {
                g.context == *context
                    && g.capability == capability
                    && g.scope == scope
                    && g.expires_ms > now
            })?
            .expires_ms;
        if capability.needs_gesture() {
            // The fresh, single-use gesture authorizes starting the operation.
            // An OS selection dialog may legitimately outlast that gesture's TTL.
            self.gesture_valid(context, gesture, now).then_some(expiry)
        } else {
            Some(expiry)
        }
    }
    pub(crate) fn consume(
        &mut self,
        context: &Context,
        capability: Capability,
        resource: &Resource,
        gesture: Option<&GestureToken>,
        now: u64,
    ) -> Result<Decision, CapabilityError> {
        let decision = self.authorize(context, capability, resource, gesture, now);
        match decision {
            Decision::Deny => return Err(CapabilityError::Denied),
            Decision::Ask => return Err(CapabilityError::NeedsConsent),
            _ => {}
        }
        let scope = resource.scope(capability)?;
        if decision == Decision::AllowOnce
            && let Some(index) = self.grants.iter().position(|g| {
                g.context == *context
                    && g.capability == capability
                    && g.scope == scope
                    && g.expires_ms > now
                    && g.duration == Duration::Once
            })
        {
            self.grants.remove(index);
        }
        if capability.needs_gesture()
            && let Some(g) = gesture
        {
            self.gestures.remove(&g.id);
        }
        Ok(decision)
    }
    pub fn revoke_instance(&mut self, context: &Context) {
        self.revocation_revision = self.revocation_revision.saturating_add(1);
        self.declared.remove(context);
        self.grants.retain(|g| g.context != *context);
        self.prompting.remove(context);
        self.denied.retain(|(owner, _, _)| owner != context);
        self.gestures.retain(|_, (owner, _)| owner != context);
    }
}
