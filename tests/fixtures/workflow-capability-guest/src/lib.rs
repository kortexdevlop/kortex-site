//! Signed integration fixture only. No production registration or alternative executor.
use kortex_sdk::*;
use serde_json::json;
#[derive(Default)]struct Guest{host:Host,read:bool,double:bool,key:String}
impl Tool for Guest{
 fn handle(&mut self,r:Request)->Reply{
  if r.kind==RequestKind::Render{return success(r.request_id,json!({"type":"text","key":"fixture","props":{"text":"Fixture signée"},"children":[]}).as_object().unwrap().clone())}let mut result=String::new();
  if r.kind==RequestKind::ActionInvoke{
   let action=r.payload["action_id"].as_str().unwrap_or("");let input=&r.payload["input"];self.read=action=="read";self.double=action=="double";self.key=input["key"].as_str().unwrap_or("").into();
   let key=if action=="escape"{"unapproved"}else{&self.key};let payload=if self.read{json!({"key":key})}else{json!({"key":key,"value":input["text"]})};
   let request=CapabilityRequest{request_id:r.request_id.clone(),capability:if self.read{"storage.read"}else{"storage.write"}.into(),payload:payload.as_object().unwrap().clone()};if self.host.enqueue(request).is_err(){return failure(r.request_id,"PERMISSION")}
  }
  if r.kind==RequestKind::CapabilityResult{if r.payload["ok"]!=true{return failure(r.request_id,"PERMISSION")}result=if self.read{r.payload["data"]["value"].as_str().unwrap_or("").into()}else{"written".into()};if self.double{self.double=false;let _=self.host.enqueue(CapabilityRequest{request_id:"00000000-0000-4000-8000-000000000002".into(),capability:"storage.write".into(),payload:json!({"key":self.key,"value":"overwritten"}).as_object().unwrap().clone()});}}
  success(r.request_id,json!({"result":result}).as_object().unwrap().clone())
 }
 fn host(&mut self)->Option<&mut Host>{Some(&mut self.host)}
}
kortex_sdk::export_tool!(Guest);
