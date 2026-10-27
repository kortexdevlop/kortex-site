use tool_calculateur_hash::{HashTool,sha256_bytes,CHUNK};
use kortex_sdk::{Host,Dispatcher};
use serde_json::{Value,json};
use base64::{Engine as _,engine::general_purpose::STANDARD};
use std::sync::Mutex;
static CALLS:Mutex<Vec<Value>>=Mutex::new(Vec::new());
fn transport(bytes:&[u8])->i32{CALLS.lock().unwrap().push(serde_json::from_slice(bytes).unwrap());0}
fn call(tool:&mut Dispatcher<HashTool>,kind:&str,payload:Value)->Value{serde_json::from_slice(&tool.dispatch(&serde_json::to_vec(&json!({"v":1,"request_id":"12010000-0000-4000-8000-ffffffffffff","kind":kind,"payload":payload})).unwrap())).unwrap()}
fn event(tool:&mut Dispatcher<HashTool>,key:&str){assert_eq!(call(tool,"ui.event",json!({"node_key":key,"event_type":"click","value":null}))["ok"],true);}
fn next()->Value{CALLS.lock().unwrap().remove(0)}
fn result(tool:&mut Dispatcher<HashTool>,request:&Value,data:Value){assert_eq!(call(tool,"capability.result",json!({"capability_request_id":request["request_id"],"ok":true,"data":data}))["ok"],true);}
#[test]fn known_vectors_and_strict_base64(){for (data,hex) in [("","e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),("YWJj","ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")]{assert_eq!(sha256_bytes(&json!({"data_base64":data})).unwrap()["hex"],hex);}for input in [json!({"data_base64":"YQ"}),json!({"data_base64":"!!!!"}),json!({"data_base64":"","extra":1}),json!({"data_base64":"A".repeat(1048004)})]{assert!(sha256_bytes(&input).is_err());}}
#[test]fn chunks_cancel_denial_and_protocol_bounds(){let mut tool=Dispatcher::new(HashTool::with_host(Host::new(transport)));assert_eq!(call(&mut tool,"init",json!({}))["ok"],true);let tree=call(&mut tool,"render",json!({}))["data"].clone();kortex_ui::ValidatedUiTree::validate(tree).unwrap();
 event(&mut tool,"pick");let pick=next();result(&mut tool,&pick,json!({"handle":"a".repeat(64),"name":"known.bin","size":CHUNK+3}));let read=next();assert_eq!(read["payload"]["length"],CHUNK);result(&mut tool,&read,json!({"data_base64":STANDARD.encode(vec![42;CHUNK]),"offset":0,"eof":false}));let read=next();assert_eq!(read["payload"]["offset"],CHUNK);result(&mut tool,&read,json!({"data_base64":STANDARD.encode(b"abc"),"offset":CHUNK,"eof":true}));let tree=call(&mut tool,"render",json!({}))["data"].clone();kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();use sha2::{Digest,Sha256};let mut bytes=vec![42;CHUNK];bytes.extend(b"abc");assert!(tree.to_string().contains(&format!("{:x}",Sha256::digest(bytes))));
 event(&mut tool,"pick");let pick=next();result(&mut tool,&pick,json!({"handle":"b".repeat(64),"name":"cancel.bin","size":100}));let pending=next();event(&mut tool,"cancel");result(&mut tool,&pending,json!({"data_base64":"YWJj","offset":0,"eof":true}));assert!(CALLS.lock().unwrap().is_empty());assert!(call(&mut tool,"render",json!({})).to_string().contains("Calcul annulé"));
 event(&mut tool,"pick");let pick=next();call(&mut tool,"capability.result",json!({"capability_request_id":pick["request_id"],"ok":false,"error":{"code":"PERMISSION","message":"PERMISSION","retryable":false}}));assert!(CALLS.lock().unwrap().is_empty());assert!(call(&mut tool,"render",json!({})).to_string().contains("PERMISSION"));
 event(&mut tool,"pick");let pick=next();result(&mut tool,&pick,json!({"handle":"c".repeat(64),"name":"invalid.bin","size":3}));let read=next();result(&mut tool,&read,json!({"data_base64":"YWJj","offset":1,"eof":true}));assert!(call(&mut tool,"render",json!({})).to_string().contains("INVALID_CHUNK"));
}
