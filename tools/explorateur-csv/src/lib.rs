pub mod parser;
pub mod stats;
mod ui;
use base64::{Engine as _,engine::general_purpose::STANDARD};
use kortex_sdk::{Reply,Request,RequestKind,Tool,Host,CapabilityRequest,success,failure};
use kortex_sdk::capability::CapabilityOutcome;
use serde_json::{Value,json};
use std::sync::{Arc,Mutex};
use parser::Table;
use stats::{Analyzer,ColumnStats};
pub const FILE_LIMIT:u64=10*1024*1024;
pub const CHUNK:usize=256*1024;
pub const ACTION_LIMIT:usize=1_000_000;
pub const PAGE_SIZE:usize=50;
const PREFS_KEY:&str="prefs/recent";
fn analyse(input:&Value)->Result<(Table,Vec<ColumnStats>),&'static str>{
 if input.as_object().is_none_or(|o|o.len()!=1){return Err("INVALID_INPUT");}
 let text=input["data_text"].as_str().ok_or("INVALID_INPUT")?;if text.len()>ACTION_LIMIT{return Err("TOO_LARGE");}
 let mut analyzer=Analyzer::new();analyzer.feed(text.as_bytes())?;analyzer.finish()
}
/// Headless action: the UI calls the very same parser and statistics code.
pub fn summarize_csv(input:&Value)->Result<Value,&'static str>{let (table,stats)=analyse(input)?;Ok(json!({"rows":table.rows(),"columns":table.columns(),"stats":stats::to_json(&stats)}))}
pub fn export_report(input:&Value)->Result<Value,&'static str>{let (_,stats)=analyse(input)?;Ok(json!({"report_csv":stats::report_csv(&stats)}))}
#[derive(Clone,Copy)]
pub(crate) enum Kind{Pick,Read{offset:u64},Save,PrefsRead,PrefsWrite}
struct Message{generation:u64,kind:Kind,outcome:CapabilityOutcome}
pub struct CsvTool{pub(crate) host:Host,inbox:Arc<Mutex<Vec<Message>>>,generation:u64,sequence:u64,running:bool,
 pub(crate) phase:&'static str,handle:String,pub(crate) name:String,pub(crate) size:u64,pub(crate) offset:u64,analyzer:Option<Analyzer>,
 pub(crate) table:Option<Table>,pub(crate) stats:Vec<ColumnStats>,pub(crate) error:String,pub(crate) notice:String,
 pub(crate) tab:String,pub(crate) page:usize,pub(crate) query:String,pub(crate) search_col:usize,pub(crate) chart_col:usize,pub(crate) chart_kind:String,pub(crate) recent:Vec<String>}
impl Default for CsvTool{fn default()->Self{Self{host:Host::default(),inbox:Arc::default(),generation:0,sequence:0,running:false,phase:"idle",handle:String::new(),name:String::new(),size:0,offset:0,analyzer:None,table:None,stats:Vec::new(),error:String::new(),notice:String::new(),tab:"apercu".into(),page:0,query:String::new(),search_col:0,chart_col:0,chart_kind:"bar".into(),recent:Vec::new()}}}
pub(crate) fn explain(code:&str)->&'static str{match code{
 "INVALID_UTF8"=>"Le fichier n’est pas de l’UTF-8 valide.","MALFORMED_QUOTE"=>"Guillemet mal placé : une valeur entre guillemets doit les ouvrir et les fermer entièrement.",
 "UNTERMINATED_QUOTE"=>"Un guillemet ouvert n’est jamais refermé.","RAGGED_ROW"=>"Une ligne n’a pas le même nombre de colonnes que l’en-tête.",
 "TOO_LARGE"=>"Le fichier dépasse 5 000 lignes, 50 colonnes ou la taille permise.","EMPTY"=>"Le fichier est vide.","INVALID_CHARACTER"=>"Le fichier contient un caractère de contrôle interdit.",
 "INVALID_LINE_ENDING"=>"Fin de ligne invalide : un retour chariot seul est refusé.","NON_FINITE_NUMBER"=>"Une valeur numérique dépasse les nombres représentables.","NUMERIC_OVERFLOW"=>"La somme d’une colonne dépasse les nombres représentables.",
 "PERMISSION"=>"Autorisation refusée ; aucun octet n’a été lu ou écrit.","NO_DATA"=>"Aucun fichier analysé.","BUSY"=>"Une opération est déjà en cours.",_=>"Opération impossible."}}
impl CsvTool{
 pub fn with_host(host:Host)->Self{Self{host,..Self::default()}}
 fn enqueue(&mut self,kind:Kind,capability:&str,payload:Value)->Result<(),&'static str>{
  self.sequence=self.sequence.checked_add(1).filter(|n|*n<1<<48).ok_or("REQUEST_LIMIT")?;let id=format!("12030000-0000-4000-8000-{:012x}",self.sequence);
  self.host.enqueue(CapabilityRequest{request_id:id.clone(),capability:capability.into(),payload:payload.as_object().unwrap().clone()}).map_err(|_|"CAPABILITY_DENIED")?;
  let inbox=self.inbox.clone();let generation=self.generation;self.host.on_result(&id,move |outcome|{if let Ok(mut queue)=inbox.lock(){queue.push(Message{generation,kind,outcome});}}).map_err(|_|"CAPABILITY_ERROR")?;Ok(())
 }
 fn fail(&mut self,code:&str){self.phase="error";self.error=code.into();self.clear_file();self.table=None;self.stats.clear();}
 fn clear_file(&mut self){self.handle.clear();self.analyzer=None;}
 fn reset(&mut self){self.generation=self.generation.wrapping_add(1);self.clear_file();self.name.clear();self.size=0;self.offset=0;self.table=None;self.stats.clear();self.error.clear();self.notice.clear();self.page=0;self.query.clear();self.search_col=0;self.chart_col=0;}
 fn busy(&self)->bool{self.host.pending_count()!=0}
 fn start(&mut self)->Result<(),&'static str>{if self.busy(){return Err("BUSY");}self.reset();self.phase="picking";self.enqueue(Kind::Pick,"file.pick",json!({}))}
 fn read(&mut self)->Result<(),&'static str>{self.enqueue(Kind::Read{offset:self.offset},"file.read",json!({"handle":self.handle,"offset":self.offset,"length":CHUNK}))}
 fn finish(&mut self){self.handle.clear();
  match self.analyzer.take().ok_or("CANCELLED").and_then(Analyzer::finish){Ok((t,s))=>{self.chart_col=s.iter().position(|c|c.kind==stats::Kind::Number).unwrap_or(0);self.table=Some(t);self.stats=s;self.phase="done";},Err(e)=>self.fail(e)}}
 fn accept(&mut self,kind:Kind,value:Value)->Result<(),&'static str>{match kind{
  Kind::Pick=>{if value["cancelled"]==true{self.reset();self.phase="cancelled";return Ok(());}
   let handle=value["handle"].as_str().filter(|s|s.len()==64&&s.bytes().all(|c|c.is_ascii_hexdigit())).ok_or("INVALID_HANDLE")?;
   self.size=value["size"].as_u64().filter(|s|*s<=FILE_LIMIT).ok_or("TOO_LARGE")?;self.handle=handle.into();self.analyzer=Some(Analyzer::new());self.name=value["name"].as_str().unwrap_or("Fichier local").chars().filter(|c|!c.is_control()).take(256).collect();self.phase="reading";
   if self.size==0{self.finish();Ok(())}else{self.read()}},
  Kind::Read{offset}=>{if self.phase!="reading"||value["offset"].as_u64()!=Some(offset)||offset!=self.offset{return Err("INVALID_CHUNK");}
   let text=value["data_base64"].as_str().filter(|s|s.len()<=349528).ok_or("INVALID_CHUNK")?;let bytes=STANDARD.decode(text).map_err(|_|"INVALID_BASE64")?;
   let next=self.offset.checked_add(bytes.len() as u64).ok_or("TOO_LARGE")?;
   if bytes.is_empty()||bytes.len()>CHUNK||next>self.size||next>FILE_LIMIT||value["eof"].as_bool()!=Some(next==self.size){return Err("INVALID_CHUNK");}
   self.analyzer.as_mut().ok_or("CANCELLED")?.feed(&bytes)?;self.offset=next;if next==self.size{self.finish();Ok(())}else{self.read()}},
  Kind::Save=>{self.notice=if value["saved"]==true{"Rapport enregistré localement.".into()}else{"Enregistrement non confirmé.".into()};Ok(())},
  Kind::PrefsRead=>{if value["value"].is_null(){self.recent.clear();self.notice="Aucun fichier récent mémorisé.".into();return Ok(());}
   let list=value["value"]["recent"].as_array().filter(|a|a.len()<=5).ok_or("INVALID_PREFERENCES")?;let mut recent=Vec::new();
   for item in list{recent.push(item.as_str().filter(|s|s.len()<=256&&!s.chars().any(char::is_control)).ok_or("INVALID_PREFERENCES")?.to_string());}
   self.recent=recent;self.notice=format!("{} fichier(s) récent(s) chargé(s).",self.recent.len());Ok(())},
  Kind::PrefsWrite=>{self.notice="Nom du fichier mémorisé sur cet appareil.".into();Ok(())}
 }}
 fn settle(&mut self){let messages=std::mem::take(&mut *self.inbox.lock().unwrap_or_else(|e|e.into_inner()));for message in messages{if message.generation!=self.generation{continue;}
  match message.outcome{Ok(data)=>{if let Err(e)=self.accept(message.kind,Value::Object(data)){if matches!(message.kind,Kind::Save|Kind::PrefsRead|Kind::PrefsWrite){self.notice=explain(e).into();}else{self.fail(e);}}},
   Err(e)=>{if e.code=="CANCELLED"{self.reset();self.phase="cancelled";}else if matches!(message.kind,Kind::Save|Kind::PrefsRead|Kind::PrefsWrite){self.notice=format!("{} ({})",explain(&e.code),e.code);}else{self.fail(&e.code);}}}}}
 fn export(&mut self)->Result<(),&'static str>{if self.busy(){return Err("BUSY");}if self.table.is_none(){return Err("NO_DATA");}let report=stats::report_csv(&self.stats);
  self.notice.clear();self.enqueue(Kind::Save,"file.save",json!({"name":"rapport-csv.csv","data_base64":STANDARD.encode(report.as_bytes())}))}
 fn prefs_read(&mut self)->Result<(),&'static str>{if self.busy(){return Err("BUSY");}self.notice.clear();self.enqueue(Kind::PrefsRead,"storage.read",json!({"key":PREFS_KEY}))}
 fn prefs_write(&mut self)->Result<(),&'static str>{if self.busy(){return Err("BUSY");}if self.table.is_none(){return Err("NO_DATA");}
  let mut recent=vec![self.name.clone()];recent.extend(self.recent.iter().filter(|n|**n!=self.name).cloned());recent.truncate(5);self.recent=recent.clone();self.notice.clear();
  self.enqueue(Kind::PrefsWrite,"storage.write",json!({"key":PREFS_KEY,"operation":"set","value":{"recent":recent}}))}
 fn event(&mut self,p:&Value)->Result<(),&'static str>{let key=p["node_key"].as_str().unwrap_or("");let kind=p["event_type"].as_str().unwrap_or("");let columns=self.table.as_ref().map_or(0,|t|t.columns());
  match (kind,key){
   ("click","pick")=>{if let Err(e)=self.start(){self.fail(e);}},
   ("click","cancel")=>{self.reset();self.phase="cancelled";},
   ("click","prev")=>self.page=self.page.saturating_sub(1),("click","next")=>self.page=self.page.saturating_add(1),
   ("click","export")=>{if let Err(e)=self.export(){self.notice=explain(e).into();}},
   ("click","prefs_load")=>{if let Err(e)=self.prefs_read(){self.notice=explain(e).into();}},
   ("click","prefs_save")=>{if let Err(e)=self.prefs_write(){self.notice=explain(e).into();}},
   ("input","search")=>{self.query=p["value"].as_str().filter(|s|s.chars().count()<=256).ok_or("INVALID_INPUT")?.into();self.page=0;},
   ("change","search_col")=>{self.search_col=p["value"].as_str().and_then(|s|s.parse::<usize>().ok()).filter(|n|*n<columns).ok_or("INVALID_INPUT")?;self.page=0;},
   ("change","chart_col")=>{self.chart_col=p["value"].as_str().and_then(|s|s.parse::<usize>().ok()).filter(|n|*n<columns&&self.stats[*n].kind==stats::Kind::Number).ok_or("INVALID_INPUT")?;},
   ("change","chart_kind")=>{self.chart_kind=p["value"].as_str().filter(|s|["bar","line"].contains(s)).ok_or("INVALID_INPUT")?.into();},
   ("select","tabs")=>{self.tab=p["value"].as_str().filter(|s|["apercu","stats","graphique"].contains(s)).ok_or("INVALID_INPUT")?.into();},
   _=>return Err("INVALID_INPUT")
  }Ok(())}
}
impl Tool for CsvTool {
 fn host(&mut self)->Option<&mut Host>{Some(&mut self.host)}
 fn handle(&mut self,r:Request)->Reply{let id=r.request_id;let p=Value::Object(r.payload);let result=match r.kind{
  RequestKind::Init=>{let host=std::mem::take(&mut self.host);*self=Self{host,running:true,..Self::default()};Ok(json!({}))},RequestKind::Shutdown=>{self.reset();self.running=false;self.phase="idle";Ok(json!({}))},_ if !self.running=>Err("NOT_INITIALIZED"),
  RequestKind::Render=>Ok(self.render()),RequestKind::CapabilityResult=>{self.settle();Ok(json!({}))},
  RequestKind::ActionInvoke=>match p["action_id"].as_str(){Some("summarize_csv")=>summarize_csv(&p["input"]),Some("export_report")=>export_report(&p["input"]),_=>Err("UNKNOWN_ACTION")},
  RequestKind::UiEvent=>self.event(&p).map(|_|json!({})),_=>Err("UNSUPPORTED_REQUEST")
 };match result{Ok(v)=>success(id,v.as_object().unwrap().clone()),Err(e)=>failure(id,e)}}
}
kortex_sdk::export_tool!(CsvTool);
