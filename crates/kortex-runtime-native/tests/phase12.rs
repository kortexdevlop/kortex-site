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
#[test]fn signed_case_utc_hash_and_csv_guests_run_identical_ui_and_actions_in_wasmtime(){let mut runtime=Runtime::new(NativeGuestEngine,TestClock);for (name,action,input,expected) in [("convertisseur-casse","upper",json!({"text":"été ß"}),json!({"text":"ÉTÉ SS"})),("timestamp-utc","from_unix",json!({"value":-1,"unit":"milliseconds"}),json!({"iso_utc":"1969-12-31T23:59:59.999Z","unix_seconds":-1,"unix_millis":-1})),("calculateur-hash","sha256_bytes",json!({"data_base64":"YWJj"}),json!({"hex":"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"})),("explorateur-csv","summarize_csv",json!({"data_text":"a,b
1,x
3,
"}),json!({"rows":2,"columns":2,"stats":[{"name":"a","kind":"number","count":2,"nulls":0,"min":1.0,"max":3.0,"mean":2.0},{"name":"b","kind":"text","count":1,"nulls":1}]}))]{let id=run(runtime.load(load(name))).unwrap();run(runtime.initialize(id)).unwrap();let reply=run(runtime.invoke_action(id,action,input)).unwrap();assert!(reply.ok);assert_eq!(Value::Object(reply.data.unwrap()),expected);let tree=Value::Object(run(runtime.render(id)).unwrap().data.unwrap());assert!(tree["children"].is_array());run(runtime.shutdown(id)).unwrap();}}
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
#[test]fn real_windows_csv_stream_stats_export_denial_and_persistent_recent_files(){
 let dir=root().join(format!("target/phase12/native-csv-{}-{}",std::process::id(),now_ms()));std::fs::create_dir_all(&dir).unwrap();let path=dir.join("données.csv");
 let mut text=String::from("id,nom,valeur
");for i in 0..4000{text.push_str(&format!("{i},\"ligne {i}, x {}\",{}
","y".repeat(200),i*3));}let bytes=text.into_bytes();assert!(bytes.len()>2*256*1024);std::fs::write(&path,&bytes).unwrap();
 let host=NativeHost::new(NativeStorage::new(dir.join("storage")).unwrap(),NativeNetwork::default());let files:NativeFiles=host.files.clone();let mut services=NativeServices::new(host);let inbox=CapabilityInbox::default();let mut runtime=Runtime::new(HostedNativeEngine::new(inbox.clone()),TestClock);
 let id=run(runtime.load(load("explorateur-csv"))).unwrap();let owner=runtime.prepare_capabilities(id,&mut services.gateway).unwrap();run(runtime.initialize(id)).unwrap();
 run(runtime.dispatch_event(id,json!({"node_key":"pick","event_type":"click","value":null}))).unwrap();let pick=inbox.pop().unwrap();assert_eq!(pick.capability,"file.pick");
 let selected=files.select(&owner,Some(&path),now_ms()).unwrap();let backend=selected["backend"].as_str().unwrap();let handle=services.gateway.handles.issue(owner.clone(),Access::Read,backend.into(),now_ms(),300000).unwrap();inbox.complete(&pick.request_id);
 run(runtime.dispatch_capability_result(id,pick.request_id,Ok(json!({"handle":handle,"name":"données.csv","size":bytes.len()}).as_object().unwrap().clone()))).unwrap();
 let mut chunks=0;let mut granted=false;while let Some(request)=inbox.pop(){assert_eq!(request.capability,"file.read");let outcome=if !granted{granted=true;run(services.fulfill(&owner,&request,true,Duration::Session))}else{run(services.execute_existing(&owner,&request))};inbox.complete(&request.request_id);run(runtime.dispatch_capability_result(id,request.request_id,outcome)).unwrap();chunks+=1;assert!(chunks<=8);}
 assert_eq!(chunks,bytes.len().div_ceil(256*1024));let tree=Value::Object(run(runtime.render(id)).unwrap().data.unwrap()).to_string();assert!(tree.contains("4000 ligne(s) sur 4000 · page 1 / 80")&&tree.contains("ligne 49, x")&&!tree.contains("ligne 50, x"));
 let expected=run(runtime.invoke_action(id,"summarize_csv",json!({"data_text":String::from_utf8(bytes.clone()).unwrap().chars().take(0).collect::<String>()+"id,v
1,2
"}))).unwrap();assert!(expected.ok);
 run(runtime.dispatch_event(id,json!({"node_key":"export","event_type":"click","value":null}))).unwrap();let save=inbox.pop().unwrap();assert_eq!(save.capability,"file.save");let denied=run(services.fulfill(&owner,&save,false,Duration::Session));assert_eq!(denied.as_ref().unwrap_err().code,"PERMISSION");inbox.complete(&save.request_id);run(runtime.dispatch_capability_result(id,save.request_id,denied)).unwrap();
 assert!(Value::Object(run(runtime.render(id)).unwrap().data.unwrap()).to_string().contains("Autorisation refusée"));
 run(runtime.dispatch_event(id,json!({"node_key":"prefs_save","event_type":"click","value":null}))).unwrap();let write=inbox.pop().unwrap();assert_eq!(write.capability,"storage.write");let outcome=run(services.fulfill(&owner,&write,true,Duration::Session));assert!(outcome.is_ok());inbox.complete(&write.request_id);run(runtime.dispatch_capability_result(id,write.request_id,outcome)).unwrap();
 run(runtime.shutdown_with_capabilities(id,&mut services.gateway)).unwrap();services.revoke(&owner);
 let id=run(runtime.load(load("explorateur-csv"))).unwrap();let owner=runtime.prepare_capabilities(id,&mut services.gateway).unwrap();run(runtime.initialize(id)).unwrap();
 run(runtime.dispatch_event(id,json!({"node_key":"prefs_load","event_type":"click","value":null}))).unwrap();let read=inbox.pop().unwrap();assert_eq!(read.capability,"storage.read");let denied=run(services.fulfill(&owner,&read,false,Duration::Session));assert_eq!(denied.as_ref().unwrap_err().code,"PERMISSION");inbox.complete(&read.request_id);run(runtime.dispatch_capability_result(id,read.request_id,denied)).unwrap();
 run(runtime.dispatch_event(id,json!({"node_key":"prefs_load","event_type":"click","value":null}))).unwrap();let read=inbox.pop().unwrap();let outcome=run(services.fulfill(&owner,&read,true,Duration::Session));assert!(outcome.is_ok());inbox.complete(&read.request_id);run(runtime.dispatch_capability_result(id,read.request_id,outcome)).unwrap();
 assert!(Value::Object(run(runtime.render(id)).unwrap().data.unwrap()).to_string().contains("Récents : données.csv"));run(runtime.shutdown_with_capabilities(id,&mut services.gateway)).unwrap();services.revoke(&owner);std::fs::remove_file(path).unwrap();
}
fn same(a:&Value,b:&Value)->bool{match(a,b){(Value::Number(x),Value::Number(y))=>x.as_f64()==y.as_f64(),(Value::Array(x),Value::Array(y))=>x.len()==y.len()&&x.iter().zip(y).all(|(p,q)|same(p,q)),(Value::Object(x),Value::Object(y))=>x.len()==y.len()&&x.iter().all(|(k,v)|y.get(k).is_some_and(|w|same(v,w))),_=>a==b}}
fn load_any(id:&str)->VerifiedTool{if root().join(format!("target/phase12/{id}/release-report.json")).exists(){return load(id);}let report:Value=serde_json::from_slice(&std::fs::read(root().join(if id=="calculatrice" && root().join("target/desktop-calculator/calculatrice/release-report.json").exists(){"target/desktop-calculator/calculatrice/release-report.json".to_string()}else{format!("target/staging/{id}/release-report.json")})).unwrap()).unwrap();let archive=std::fs::read(report["archive"].as_str().unwrap()).unwrap();let descriptor=kortex_verify::Descriptor::parse(&std::fs::read(report["descriptor"].as_str().unwrap()).unwrap()).unwrap();let trust:HostTrust=serde_json::from_slice(&std::fs::read(report["host_trust"].as_str().unwrap()).unwrap()).unwrap();verify_package_with_trust(archive,&descriptor,&trust,now_ms()).unwrap()}
#[test]fn five_tools_produce_the_shared_web_vectors_on_the_native_wasmtime_host(){
 let vectors:Vec<Value>=serde_json::from_slice(&std::fs::read(root().join("tests/integration/five-tools-vectors.json")).unwrap()).unwrap();let mut runtime=Runtime::new(NativeGuestEngine,TestClock);let mut tools=std::collections::BTreeSet::new();
 for vector in &vectors{let name=vector["tool"].as_str().unwrap();tools.insert(name.to_string());let id=run(runtime.load(load_any(name))).unwrap();run(runtime.initialize(id)).unwrap();
  let reply=run(runtime.invoke_action(id,vector["action"].as_str().unwrap(),vector["input"].clone())).unwrap();assert!(reply.ok,"{name} {}",vector["action"]);let output=Value::Object(reply.data.unwrap());assert!(same(&output,&vector["output"]),"{name} {}: {output} != {}",vector["action"],vector["output"]);run(runtime.shutdown(id)).unwrap();}
 assert_eq!(tools.len(),5);
}
