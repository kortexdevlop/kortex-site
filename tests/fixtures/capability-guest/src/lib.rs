use kortex_sdk::*;
#[derive(Default)]
struct Guest {
    host: Host,
    outcome: String,
}
impl Tool for Guest {
    fn handle(&mut self, r: Request) -> Reply {
        if r.kind == RequestKind::Init {
            let request = CapabilityRequest {
                request_id: "00000000-0000-4000-8000-000000000002".into(),
                capability: "storage.read".into(),
                payload: serde_json::json!({"key":"prefs"})
                    .as_object()
                    .unwrap()
                    .clone(),
            };
            self.outcome = if self.host.enqueue(request).is_ok() {
                "pending"
            } else {
                "enqueue_failed"
            }
            .into();
        }
        if r.kind == RequestKind::CapabilityResult {
            self.outcome = if r.payload["ok"] == true {
                format!("resolved:{}", r.payload["data"]["value"])
            } else {
                r.payload["error"]["code"]
                    .as_str()
                    .unwrap_or("ERROR")
                    .into()
            };
        }
        let value = if r.kind == RequestKind::Render {
            serde_json::json!({"type":"text","key":"outcome","props":{"text":self.outcome},"children":[]})
        } else {
            serde_json::json!({"result":self.outcome})
        };
        success(r.request_id, value.as_object().unwrap().clone())
    }
    fn host(&mut self) -> Option<&mut Host> {
        Some(&mut self.host)
    }
}
kortex_sdk::export_tool!(Guest);
