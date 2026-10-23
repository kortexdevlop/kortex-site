use crate::{Capability, CapabilityError};
use std::collections::VecDeque;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Allowed,
    Denied,
    NeedsConsent,
    Failed,
}
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub at_ms: u64,
    pub instance_id: u64,
    pub capability: Capability,
    pub outcome: Outcome,
    pub bytes_in: usize,
}
/// Local bounded journal. No resource name, path, URL, token, payload, or backend error text.
#[derive(Default)]
pub struct AuditLog {
    events: VecDeque<AuditEvent>,
}
impl AuditLog {
    pub(crate) fn record(&mut self, event: AuditEvent) {
        if self.events.len() == 1024 {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }
    pub fn events(&self) -> impl Iterator<Item = &AuditEvent> {
        self.events.iter()
    }
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
pub(crate) fn outcome<T>(result: &Result<T, CapabilityError>) -> Outcome {
    match result {
        Ok(_) => Outcome::Allowed,
        Err(CapabilityError::NeedsConsent) => Outcome::NeedsConsent,
        Err(
            CapabilityError::Denied
            | CapabilityError::InvalidHandle
            | CapabilityError::NetworkDenied,
        ) => Outcome::Denied,
        Err(_) => Outcome::Failed,
    }
}
