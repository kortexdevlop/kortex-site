use kortex_sdk::{Tool,Request,Reply,RequestKind,success,failure};
use serde_json::{json,Value};
#[derive(Default)]pub struct Echo;
impl Tool for Echo {fn handle(&mut self,r:Request)->Reply {let id=r.request_id;let p=Value::Object(r.payload);let out=match r.kind {
RequestKind::Init|RequestKind::Shutdown=>json!({}),
RequestKind::Render=>json!({"type":"text","key":"echo","props":{"text":"Extension temporaire prête"},"children":[]}),
RequestKind::ActionInvoke if p["action_id"]=="echo" && p["input"]["text"].as_str().is_some_and(|v|v.chars().count()<=256)=>json!({"text":p["input"]["text"]}),
_=>return failure(id,"INVALID_INPUT")};success(id,out.as_object().unwrap().clone())}}
kortex_sdk::export_tool!(Echo);
