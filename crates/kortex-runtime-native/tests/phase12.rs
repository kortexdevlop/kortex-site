mod common;
use common::run;
use kortex_core::{Runtime,VerifiedTool,dispatcher::Clock};
use kortex_runtime_native::{NativeGuestEngine,HostedNativeEngine,HostTrust,verify_package_with_trust,permissions::{CapabilityInbox,NativeServices,NativeHost,now_ms},files::NativeFiles,storage::NativeStorage,network::NativeNetwork};
use kortex_capabilities::{handles::Access,policy::Duration};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
struct TestClock;
impl Clock for TestClock{fn now_ms(&self)->u64{now_ms()}async fn wait_until(&self,deadline:u64){std::future::poll_fn(|cx|{if now_ms()>=deadline{std::task::Poll::Ready(())}else{cx.waker().wake_by_ref();std::task::Poll::Pending}}).await}}
fn root()->std::path::PathBuf{std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")}
fn load(id:&str)->VerifiedTool{let report:Value=serde_json::from_slice(&std::fs::read(root().join(format!("target/phase12/{id}/release-report.json"))).unwrap()).unwrap();let archive=std::fs::read(report["archive"].as_str().unwrap()).unwrap();let descriptor=kortex_verify::Descriptor::parse(&std::fs::read(report["descriptor"].as_str().unwrap()).unwrap()).unwrap();let trust:HostTrust=serde_json::from_slice(&std::fs::read(report["host_trust"].as_str().unwrap()).unwrap()).unwrap();verify_package_with_trust(archive,&descriptor,&trust,now_ms()).unwrap()}
#[test]fn signed_case_and_utc_guests_run_identical_ui_and_actions_in_wasmtime(){let mut runtime=Runtime::new(NativeGuestEngine,TestClock);for (name,action,input,expected) in [("convertisseur-casse","upper",json!({"text":"été ß"}),json!({"text":"ÉTÉ SS"})),("timestamp-utc","from_unix",json!({"value":-1,"unit":"milliseconds"}),json!({"iso_utc":"1969-12-31T23:59:59.999Z","unix_seconds":-1,"unix_millis":-1})),("calculateur-hash","sha256_bytes",json!({"data_base64":"YWJj"}),json!({"hex":"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}))]{let id=run(runtime.load(load(name))).unwrap();run(runtime.initialize(id)).unwrap();let reply=run(runtime.invoke_action(id,action,input)).unwrap();assert!(reply.ok);assert_eq!(Value::Object(reply.data.unwrap()),expected);let tree=Value::Object(run(runtime.render(id)).unwrap().data.unwrap());assert!(tree["children"].is_array());run(runtime.shutdown(id)).unwrap();}}
#[test]fn real_windows_file_stream_permission_denial_and_cancel(){
 let dir=root().join(format!("target/phase12/native-file-{}-{}",std::process::id(),now_ms()));std::fs::create_dir_all(&dir).unwrap();let path=dir.join("known.bin");let bytes=vec![0xa5;2*256*1024+31];std::fs::write(&path,&bytes).unwrap();
 let host=NativeHost::new(NativeStorage::new(dir.join("storage")).unwrap(),NativeNetwork::default());let files:NativeFiles=host.files.clone();let mut services=NativeServices::new(host);let inbox=CapabilityInbox::default();let mut runtime=Runtime::new(HostedNativeEngine::new(inbox.clone()),TestClock);
 for mode in ["deny","success","cancel"]{
  let id=run(runtime.load(load("calculateur-hash"))).unwrap();let owner=runtime.prepare_capabilities(id,&mut services.gateway).unwrap();run(runtime.initialize(id)).unwrap();
  run(runtime.dispatch_event(id,json!({"node_key":"pick","event_type":"click","value":null}))).unwrap();let pick=inbox.pop().unwrap();assert_eq!(pick.capability,"file.pick");
  // Test-owned selection at the native picker boundary: real disk/handles, no I/O mock.
  let selected=files.select(&owner,Some(&path),now_ms()).unwrap();let backend=selected["backend"].as_str().unwrap();let handle=services.gateway.handles.issue(owner.clone(),Access::Read,backend.into(),now_ms(),300000).unwrap();inbox.complete(&pick.request_id);
  run(runtime.dispatch_capability_result(id,pick.request_id,Ok(json!({"handle":handle,"name":"known.bin","size":bytes.len()}).as_object().unwrap().clone()))).unwrap();
  let first=inbox.pop().unwrap();assert_eq!(first.capability,"file.read");assert_eq!(first.payload["length"],256*1024);
  if mode=="cancel"{run(runtime.dispatch_event(id,json!({"node_key":"cancel","event_type":"click","value":null}))).unwrap();}
  let outcome=run(services.fulfill(&owner,&first,mode=="success",Duration::Session));if mode!="success"{assert_eq!(outcome.as_ref().unwrap_err().code,"PERMISSION");}inbox.complete(&first.request_id);run(runtime.dispatch_capability_result(id,first.request_id,outcome)).unwrap();let mut chunks=1;
  while let Some(request)=inbox.pop(){assert_eq!(mode,"success");assert_eq!(request.payload["length"],256*1024);let result=run(services.execute_existing(&owner,&request));inbox.complete(&request.request_id);run(runtime.dispatch_capability_result(id,request.request_id,result)).unwrap();chunks+=1;assert!(chunks<=3);}
  let tree=Value::Object(run(runtime.render(id)).unwrap().data.unwrap()).to_string();match mode{"success"=>{assert_eq!(chunks,3);assert!(tree.contains(&format!("{:x}",Sha256::digest(&bytes))));},"deny"=>assert!(tree.contains("PERMISSION")),_=>assert!(tree.contains("Calcul annulé"))};run(runtime.shutdown_with_capabilities(id,&mut services.gateway)).unwrap();services.revoke(&owner);
 }
 std::fs::remove_file(path).unwrap();
}
