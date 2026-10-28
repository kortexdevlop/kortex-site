//! Ephemeral trusted-host UI channel. Neither decisions nor resources are serialized.
use kortex_capabilities::{Capability, Resource};
use kortex_workflows::{Error, runner::Cancellation};
use serde_json::Value;
use std::sync::{Arc, Mutex};
#[derive(Clone, Debug, PartialEq)]
pub struct Prompt {
    pub id: String,
    pub node: String,
    pub tool: String,
    pub version: String,
    pub capability: String,
    pub resource: String,
    pub editable: bool,
}
#[derive(Default)]
struct State {
    pending: Option<Prompt>,
    answer: Option<(String, bool, String)>,
}
#[derive(Clone, Default)]
pub struct Permissions {
    state: Arc<Mutex<State>>,
    interactive: bool,
}
impl Permissions {
    pub fn interactive() -> Self {
        Self {
            interactive: true,
            ..Self::default()
        }
    }
    pub fn pending(&self) -> Option<Prompt> {
        self.state.lock().ok()?.pending.clone()
    }
    pub fn respond(&self, id: &str, allow: bool, resource: String) {
        if let Ok(mut s) = self.state.lock() {
            if s.pending.as_ref().is_some_and(|p| p.id == id) && s.answer.is_none() {
                s.answer = Some((id.into(), allow, resource))
            }
        }
    }
    pub fn clear(&self) {
        if let Ok(mut s) = self.state.lock() {
            *s = State::default()
        }
    }
    pub fn ask(
        &self,
        node: &str,
        tool: &str,
        version: &str,
        capability: &str,
        resource: String,
        editable: bool,
        cancel: &Cancellation,
        deadline: u64,
    ) -> Result<(String, u64), Error> {
        if !self.interactive {
            return Err(Error::PermissionDenied);
        }
        let id = crate::paths::nonce().map_err(|_| Error::PermissionDenied)?;
        {
            let mut s = self.state.lock().map_err(|_| Error::PermissionDenied)?;
            if s.pending.is_some() {
                return Err(Error::PermissionDenied);
            }
            s.pending = Some(Prompt {
                id: id.clone(),
                node: node.into(),
                tool: tool.into(),
                version: version.into(),
                capability: capability.into(),
                resource: resource.clone(),
                editable,
            });
        }
        let result = loop {
            if cancel.is_cancelled() {
                break Err(Error::Cancelled);
            }
            let now = kortex_runtime_native::permissions::now_ms();
            if now >= deadline {
                break Err(Error::Timeout);
            }
            let answer = self
                .state
                .lock()
                .map_err(|_| Error::PermissionDenied)?
                .answer
                .take();
            if let Some((ticket, allow, value)) = answer {
                if ticket != id || !allow || value.len() > 4096 || !editable && value != resource {
                    break Err(Error::PermissionDenied);
                }
                break Ok((value, now));
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        if let Ok(mut s) = self.state.lock() {
            if s.pending.as_ref().is_some_and(|p| p.id == id) {
                *s = State::default()
            }
        }
        result
    }
}
pub fn suggested(cap: Capability, input: &Value) -> String {
    match cap {
        Capability::StorageRead | Capability::StorageWrite => {
            input["key"].as_str().unwrap_or("").into()
        }
        Capability::NetworkFetch => input["url"].as_str().unwrap_or("").into(),
        Capability::FileRead => input["handle"].as_str().unwrap_or("").into(),
        Capability::FilePick | Capability::FileSave => "Boîte de sélection de fichier".into(),
        Capability::ClipboardWrite => "Presse-papiers".into(),
    }
}
pub fn editable(cap: Capability) -> bool {
    matches!(
        cap,
        Capability::StorageRead
            | Capability::StorageWrite
            | Capability::NetworkFetch
            | Capability::FileRead
    )
}
pub fn resource(cap: Capability, value: String) -> Result<Resource, Error> {
    let r = match cap {
        Capability::StorageRead | Capability::StorageWrite => Resource::Storage { key: value },
        Capability::NetworkFetch => Resource::Network { url: value },
        Capability::FileRead => Resource::File { handle: value },
        Capability::FilePick | Capability::FileSave => Resource::FileDialog,
        Capability::ClipboardWrite => Resource::Clipboard,
    };
    r.scope(cap).map_err(|_| Error::PermissionDenied)?;
    Ok(r)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expired_or_modified_fixed_resource_never_grants() {
        let b = Permissions::interactive();
        assert!(matches!(
            b.ask(
                "n",
                "tool",
                "1.0.0",
                "file.pick",
                "dialog".into(),
                false,
                &Cancellation::default(),
                0
            ),
            Err(Error::Timeout)
        ));
        assert!(b.pending().is_none());
        assert!(resource(Capability::StorageWrite, "../escape".into()).is_err());
        assert!(resource(Capability::FileRead, "not-a-handle".into()).is_err());
    }
    #[test]
    fn tickets_are_single_use_and_cancelled_prompts_cannot_approve_a_later_run() {
        let broker = Permissions::interactive();
        let child = broker.clone();
        let cancel = Cancellation::default();
        let c = cancel.clone();
        let thread = std::thread::spawn(move || {
            child.ask(
                "node",
                "tool",
                "1.0.0",
                "storage.write",
                "prefs".into(),
                true,
                &c,
                kortex_runtime_native::permissions::now_ms() + 2000,
            )
        });
        while broker.pending().is_none() {
            std::thread::yield_now()
        }
        let p = broker.pending().unwrap();
        broker.respond("wrong", true, "prefs".into());
        assert!(broker.pending().is_some());
        cancel.cancel();
        assert!(matches!(thread.join().unwrap(), Err(Error::Cancelled)));
        broker.respond(&p.id, true, "prefs".into());
        assert!(broker.pending().is_none());
        assert!(
            Permissions::default()
                .ask(
                    "n",
                    "tool",
                    "1.0.0",
                    "file.read",
                    "".into(),
                    true,
                    &Cancellation::default(),
                    u64::MAX
                )
                .is_err()
        );
    }
}
