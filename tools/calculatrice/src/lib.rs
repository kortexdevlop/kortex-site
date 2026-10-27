//! Pure calculator guest with one arithmetic path for UI and actions.
mod state;
use kortex_sdk::{Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
pub use state::State;
pub const MANIFEST: &str = include_str!("../manifest.json");
pub const CONTRACT: &str = include_str!("../contract.json");
#[derive(Default)]
pub struct Calculator {
    state: State,
    running: bool,
}
impl Tool for Calculator {
    fn handle(&mut self, request: Request) -> Reply {
        let id = request.request_id;
        let p = Value::Object(request.payload);
        let outcome = match request.kind {
            RequestKind::Init => {
                self.state = State::default();
                self.running = true;
                Ok(json!({}))
            }
            RequestKind::Shutdown => {
                self.state = State::default();
                self.running = false;
                Ok(json!({}))
            }
            _ if !self.running => Err("NOT_INITIALIZED"),
            RequestKind::Render => Ok(self.state.render()),
            RequestKind::ActionInvoke => self
                .state
                .calculate(p["action_id"].as_str().unwrap_or(""), &p["input"]),
            RequestKind::UiEvent => self.state.event(&p).map(|()| json!({})),
            _ => Err("UNSUPPORTED_REQUEST"),
        };
        match outcome {
            Ok(data) => success(id, data.as_object().unwrap().clone()),
            Err(code) => failure(id, code),
        }
    }
}
kortex_sdk::export_tool!(Calculator);
