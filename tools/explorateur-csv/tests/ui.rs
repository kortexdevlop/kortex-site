use tool_explorateur_csv::{CsvTool,CHUNK};
use kortex_sdk::{Host,Dispatcher};
use serde_json::{Value,json};
use base64::{Engine as _,engine::general_purpose::STANDARD};
use std::sync::Mutex;
static CALLS:Mutex<Vec<Value>>=Mutex::new(Vec::new());
static SERIAL:Mutex<()>=Mutex::new(());
fn transport(bytes:&[u8])->i32{CALLS.lock().unwrap().push(serde_json::from_slice(bytes).unwrap());0}
fn call(tool:&mut Dispatcher<CsvTool>,kind:&str,payload:Value)->Value{serde_json::from_slice(&tool.dispatch(&serde_json::to_vec(&json!({"v":1,"request_id":"12030000-0000-4000-8000-ffffffffffff","kind":kind,"payload":payload})).unwrap())).unwrap()}
fn ui(tool:&mut Dispatcher<CsvTool>,key:&str,kind:&str,value:Value)->Value{call(tool,"ui.event",json!({"node_key":key,"event_type":kind,"value":value}))}
fn click(tool:&mut Dispatcher<CsvTool>,key:&str){assert_eq!(ui(tool,key,"click",Value::Null)["ok"],true,"{key}");}
fn next()->Value{CALLS.lock().unwrap().remove(0)}
fn result(tool:&mut Dispatcher<CsvTool>,request:&Value,data:Value){assert_eq!(call(tool,"capability.result",json!({"capability_request_id":request["request_id"],"ok":true,"data":data}))["ok"],true);}
fn render(tool:&mut Dispatcher<CsvTool>)->Value{let tree=call(tool,"render",json!({}))["data"].clone();kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();tree}
fn fresh()->Dispatcher<CsvTool>{CALLS.lock().unwrap().clear();let mut tool=Dispatcher::new(CsvTool::with_host(Host::new(transport)));assert_eq!(call(&mut tool,"init",json!({}))["ok"],true);tool}
fn load(tool:&mut Dispatcher<CsvTool>,name:&str,bytes:&[u8]){
 click(tool,"pick");let pick=next();assert_eq!(pick["capability"],"file.pick");result(tool,&pick,json!({"handle":"a".repeat(64),"name":name,"size":bytes.len()}));
 let mut offset=0;while offset<bytes.len(){let read=next();assert_eq!(read["capability"],"file.read");assert_eq!(read["payload"]["offset"],offset);let end=(offset+CHUNK).min(bytes.len());result(tool,&read,json!({"data_base64":STANDARD.encode(&bytes[offset..end]),"offset":offset,"eof":end==bytes.len()}));offset=end;}
}
fn find<'a>(tree:&'a Value,key:&str)->Option<&'a Value>{if tree["key"]==key{return Some(tree);}tree["children"].as_array().unwrap().iter().find_map(|c|find(c,key))}
fn csv(rows:usize)->String{let mut s=String::from("id,nom,valeur\n");for i in 0..rows{s.push_str(&format!("{i},ligne {i},{}\n",i*2));}s}
#[test]fn interactions_pagination_search_chart_and_same_action_output(){let _g=SERIAL.lock().unwrap_or_else(|e|e.into_inner());let mut tool=fresh();
 load(&mut tool,"data.csv",csv(120).as_bytes());let tree=render(&mut tool);
 assert_eq!(find(&tree,"rows").unwrap()["props"]["rows"].as_array().unwrap().len(),50);assert!(find(&tree,"range").unwrap()["props"]["text"].as_str().unwrap().contains("120 ligne(s) sur 120 · page 1 / 3"));
 click(&mut tool,"next");click(&mut tool,"next");click(&mut tool,"next");let tree=render(&mut tool);assert_eq!(find(&tree,"rows").unwrap()["props"]["rows"].as_array().unwrap().len(),20);assert_eq!(find(&tree,"next").unwrap()["props"]["disabled"],true);
 assert_eq!(ui(&mut tool,"search_col","change",json!("1"))["ok"],true);assert_eq!(ui(&mut tool,"search","input",json!("ligne 11"))["ok"],true);let tree=render(&mut tool);assert_eq!(find(&tree,"rows").unwrap()["props"]["rows"].as_array().unwrap().len(),11);
 assert_eq!(ui(&mut tool,"search_col","change",json!("9"))["ok"],false);assert_eq!(ui(&mut tool,"search_col","change",json!("0"))["ok"],true);assert_eq!(ui(&mut tool,"search","input",json!("119"))["ok"],true);assert_eq!(find(&render(&mut tool),"rows").unwrap()["props"]["rows"].as_array().unwrap().len(),1);
 assert_eq!(ui(&mut tool,"tabs","select",json!("graphique"))["ok"],true);assert_eq!(ui(&mut tool,"tabs","select",json!("autre"))["ok"],false);
 let tree=render(&mut tool);let chart=find(&tree,"chart").unwrap();assert_eq!(chart["props"]["labels"].as_array().unwrap().len(),120);assert_eq!(chart["props"]["series"][0]["label"],"id");
 assert_eq!(ui(&mut tool,"chart_col","change",json!("1"))["ok"],false);assert_eq!(ui(&mut tool,"chart_col","change",json!("2"))["ok"],true);assert_eq!(ui(&mut tool,"chart_kind","change",json!("line"))["ok"],true);
 let chart=render(&mut tool);let chart=find(&chart,"chart").unwrap().clone();assert_eq!(chart["props"]["kind"],"line");assert_eq!(chart["props"]["series"][0]["values"][119],238.0);
 let stats=find(&render(&mut tool),"stats_table").unwrap()["props"]["rows"].clone();
 let action=call(&mut tool,"action.invoke",json!({"action_id":"summarize_csv","input":{"data_text":csv(120)}}))["data"].clone();
 assert_eq!(stats[2]["max"],action["stats"][2]["max"]);assert_eq!(stats[2]["mean"],action["stats"][2]["mean"]);assert_eq!(action["rows"],120);
}
#[test]fn hostile_file_is_explained_and_leaves_no_table(){let _g=SERIAL.lock().unwrap_or_else(|e|e.into_inner());let mut tool=fresh();
 for (bytes,code) in [(&b"a,b\n1,\"2\n"[..],"UNTERMINATED_QUOTE"),(&b"a,b\n\xff,2\n"[..],"INVALID_UTF8"),(&b"a,b\n1,x\"y\n"[..],"MALFORMED_QUOTE")]{load(&mut tool,"bad.csv",bytes);let tree=render(&mut tool);assert!(find(&tree,"status").unwrap()["props"]["text"].as_str().unwrap().contains(code));assert!(find(&tree,"tabs").is_none());assert!(find(&tree,"export").is_none());}
 load(&mut tool,"big.csv",csv(5001).as_bytes());assert!(find(&render(&mut tool),"status").unwrap()["props"]["text"].as_str().unwrap().contains("TOO_LARGE"));
 click(&mut tool,"pick");let pick=next();result(&mut tool,&pick,json!({"handle":"b".repeat(64),"name":"huge.csv","size":10*1024*1024+1}));assert!(find(&render(&mut tool),"status").unwrap()["props"]["text"].as_str().unwrap().contains("TOO_LARGE"));assert!(CALLS.lock().unwrap().is_empty());
}
#[test]fn permission_denial_cancel_and_late_results_expose_nothing(){let _g=SERIAL.lock().unwrap_or_else(|e|e.into_inner());let mut tool=fresh();
 click(&mut tool,"pick");let pick=next();call(&mut tool,"capability.result",json!({"capability_request_id":pick["request_id"],"ok":false,"error":{"code":"PERMISSION","message":"PERMISSION","retryable":false}}));assert!(CALLS.lock().unwrap().is_empty());assert!(find(&render(&mut tool),"status").unwrap()["props"]["text"].as_str().unwrap().contains("PERMISSION"));
 click(&mut tool,"pick");let pick=next();result(&mut tool,&pick,json!({"handle":"c".repeat(64),"name":"x.csv","size":100}));let pending=next();click(&mut tool,"cancel");result(&mut tool,&pending,json!({"data_base64":STANDARD.encode(b"a\n1\n"),"offset":0,"eof":true}));
 assert!(CALLS.lock().unwrap().is_empty());let tree=render(&mut tool);assert!(find(&tree,"tabs").is_none());assert!(find(&tree,"status").unwrap()["props"]["text"].as_str().unwrap().contains("annulée"));
 click(&mut tool,"pick");let pick=next();result(&mut tool,&pick,json!({"handle":"d".repeat(64),"name":"x.csv","size":3}));let read=next();result(&mut tool,&read,json!({"data_base64":"YQo=","offset":1,"eof":true}));assert!(find(&render(&mut tool),"status").unwrap()["props"]["text"].as_str().unwrap().contains("INVALID_CHUNK"));
}
#[test]fn export_and_storage_use_gesture_bound_capabilities_and_report_denial(){let _g=SERIAL.lock().unwrap_or_else(|e|e.into_inner());let mut tool=fresh();
 click(&mut tool,"export");assert!(CALLS.lock().unwrap().is_empty());assert!(find(&render(&mut tool),"notice").unwrap()["props"]["text"].as_str().unwrap().contains("Aucun fichier"));
 load(&mut tool,"données.csv","a,b\n1,x\n3,y\n".as_bytes());
 click(&mut tool,"export");let save=next();assert_eq!(save["capability"],"file.save");assert_eq!(save["payload"]["name"],"rapport-csv.csv");let report=String::from_utf8(STANDARD.decode(save["payload"]["data_base64"].as_str().unwrap()).unwrap()).unwrap();assert!(report.starts_with("colonne,type,valeurs"));
 call(&mut tool,"capability.result",json!({"capability_request_id":save["request_id"],"ok":false,"error":{"code":"PERMISSION","message":"PERMISSION","retryable":false}}));assert!(find(&render(&mut tool),"notice").unwrap()["props"]["text"].as_str().unwrap().contains("Autorisation refusée"));assert!(find(&render(&mut tool),"tabs").is_some());
 click(&mut tool,"export");let save=next();result(&mut tool,&save,json!({"saved":true}));assert!(find(&render(&mut tool),"notice").unwrap()["props"]["text"].as_str().unwrap().contains("enregistré"));
 click(&mut tool,"prefs_save");let write=next();assert_eq!((write["capability"].as_str(),write["payload"]["key"].as_str(),write["payload"]["value"].clone()),(Some("storage.write"),Some("prefs/recent"),json!({"recent":["données.csv"]})));result(&mut tool,&write,json!({"written":true}));
 click(&mut tool,"prefs_load");let read=next();assert_eq!(read["capability"],"storage.read");result(&mut tool,&read,json!({"value":{"recent":["z.csv","y.csv"]}}));assert!(find(&render(&mut tool),"recent").unwrap()["props"]["text"].as_str().unwrap().contains("z.csv · y.csv"));
 click(&mut tool,"prefs_load");let read=next();result(&mut tool,&read,json!({"value":{"recent":[1]}}));assert!(find(&render(&mut tool),"notice").unwrap()["props"]["text"].as_str().unwrap().contains("Opération impossible"));
}
#[test]fn tree_stays_within_ui_limits_for_maximum_table(){let _g=SERIAL.lock().unwrap_or_else(|e|e.into_inner());let mut tool=fresh();
 let mut s=(0..50).map(|i|format!("c{i}")).collect::<Vec<_>>().join(",");s.push('\n');for r in 0..5000{s.push_str(&(0..50).map(|c|(r*c).to_string()).collect::<Vec<_>>().join(","));s.push('\n');}
 load(&mut tool,"max.csv",s.as_bytes());let tree=render(&mut tool);assert_eq!(find(&tree,"rows").unwrap()["props"]["rows"].as_array().unwrap().len(),50);assert!(find(&tree,"chart").unwrap()["props"]["labels"].as_array().unwrap().len()<=500);
}
