mod common;
use common::{tool,run,deadline};
use kortex_core::{GuestEngine,RuntimeError};
use kortex_abi::{Envelope,RequestKind,pointer};
use kortex_runtime_native::NativeGuestEngine;
use std::time::{Duration,Instant};
fn module(alloc:&str,dispatch:&str,free:&str)->Vec<u8>{wat::parse_str(format!(r#"(module
 (memory (export "memory") 1 1024)
 (func (export "kortex_alloc") (param i32) (result i32) {alloc})
 (func (export "kortex_dealloc") (param i32 i32) {free})
 (func (export "kortex_dispatch") (param i32 i32) (result i64) {dispatch})
 (data (i32.const 1024) "{{\22v\22:1,\22request_id\22:\2200000000-0000-4000-8000-000000000000\22,\22ok\22:true,\22data\22:{{}}}}"))"#)).unwrap()}
#[test]
fn hostile_alloc_dispatch_dealloc_and_overlap_never_crash_the_host(){
 let valid_reply=serde_json::to_vec(&serde_json::json!({"v":1,"request_id":"00000000-0000-4000-8000-000000000000","ok":true,"data":{}})).unwrap();
 let output=format!("i64.const {}",pointer::pack(1024,valid_reply.len() as u32));
 for (name,alloc,body,free) in [
  ("alloc loop","(loop br 0) i32.const 64",output.as_str(),""),
  ("dispatch loop","i32.const 64","(loop br 0) i64.const 0",""),
  ("dealloc loop","i32.const 64",output.as_str(),"(loop br 0)"),
  ("alloc overflow","i32.const -1",output.as_str(),""),
  ("output overflow","i32.const 64","i64.const -1",""),
  ("overlap","i32.const 64","i64.const 274877906964",""),
  ("recursive trap","unreachable",output.as_str(),""),
  ("dealloc trap","i32.const 64",output.as_str(),"unreachable"),
 ]{
  let bytes=module(alloc,body,free);let mut engine=NativeGuestEngine;let mut handle=run(engine.load(&tool(&bytes))).unwrap();
  let request=Envelope{v:1,request_id:"00000000-0000-4000-8000-000000000000".into(),kind:RequestKind::Init,payload:Default::default()};let start=Instant::now();
  assert!(run(engine.dispatch(&mut handle,&request,deadline()-29700)).is_err(),"{name}");run(engine.unload(handle)).unwrap();assert!(start.elapsed()<Duration::from_secs(5),"{name}");
 }
}
#[test]
fn table_growth_and_memory_growth_are_rejected_before_instantiation(){
 for table in ["(table 1 funcref)","(table 1 65537 funcref)","(table 0 1 funcref)(table 0 1 funcref)"]{
  let source=format!(r#"(module {table}(memory (export "memory") 1 1024)(func (export "kortex_alloc") (param i32)(result i32)i32.const 64)(func (export "kortex_dealloc")(param i32 i32))(func (export "kortex_dispatch")(param i32 i32)(result i64)i64.const 0))"#);let bytes=wat::parse_str(source).unwrap();assert!(matches!(run(NativeGuestEngine::instantiate(tool(&bytes))),Err(RuntimeError::UnsupportedAbi)));
 }
 for memory in ["1", "1 1025", "1 1024 shared"] {let source=format!(r#"(module (memory (export "memory") {memory})(func (export "kortex_alloc")(param i32)(result i32)i32.const 64)(func (export "kortex_dealloc")(param i32 i32))(func (export "kortex_dispatch")(param i32 i32)(result i64)i64.const 0))"#);assert!(kortex_pack::wasm_check::check(&wat::parse_str(source).unwrap()).is_err());}
}
#[test]
fn deterministic_pointer_mutations_fail_closed(){
 let mut seed=0x18011802u32;for _ in 0..4096{seed=seed.wrapping_mul(1664525).wrapping_add(1013904223);let ptr=seed;seed=seed.wrapping_mul(1664525).wrapping_add(1013904223);let len=seed;let actual=pointer::checked_range(ptr,len,65536);let valid=len>0&&len<=1048576&&ptr.checked_add(len).is_some_and(|n|n<=65536);assert_eq!(actual.is_ok(),valid);assert_eq!(pointer::unpack(pointer::pack(ptr,len)),(ptr,len));}
}
#[test]
fn network_backend_itself_rejects_private_and_mixed_pinned_addresses(){
 use kortex_runtime_native::network::NativeNetwork;
 use kortex_capabilities::CapabilityError;
 let host=NativeNetwork::new(["https://allowed.example".to_string()]).unwrap();
 for url in ["https://127.1","https://2130706433","https://0x7f000001","https://169.254.169.254","https://[::ffff:127.0.0.1]"] { assert!(NativeNetwork::url(url).is_err()); }
 let literal=NativeNetwork::new(["https://8.8.8.8".into()]).unwrap();assert!(matches!(literal.fetch("https://8.8.8.8",&["1.1.1.1".parse().unwrap()],Duration::from_millis(1)),Err(CapabilityError::NetworkDenied)));
 for ip in ["127.0.0.1","10.0.0.1","169.254.169.254","100.64.0.1","::1","fc00::1","::ffff:127.0.0.1"]{
  for addresses in [vec![ip.parse().unwrap()],vec!["8.8.8.8".parse().unwrap(),ip.parse().unwrap()]]{
   assert!(matches!(host.fetch("https://allowed.example/private",&addresses,Duration::from_millis(1)),Err(CapabilityError::NetworkDenied)));
  }
 }
}
