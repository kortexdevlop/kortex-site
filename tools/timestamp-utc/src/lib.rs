use kortex_sdk::{Reply,Request,RequestKind,Tool,success,failure};
use serde_json::{Value,json};
use time::{OffsetDateTime,format_description::well_known::Rfc3339};
pub fn convert(action:&str,input:&Value)->Result<Value,&'static str>{
 let object=input.as_object().ok_or("INVALID_INPUT")?;
 let millis=match action {
  "to_unix"=>{if object.len()!=1{return Err("INVALID_INPUT");}let iso=input["iso_utc"].as_str().filter(|s|s.len()<=32&&s.ends_with('Z')&&s.is_ascii()).ok_or("UTC_REQUIRED")?;
   if iso.len()<20||iso.as_bytes().get(10)!=Some(&b'T')||iso.as_bytes().get(17..19)==Some(b"60"){return Err("INVALID_DATE");}
   let date=OffsetDateTime::parse(iso,&Rfc3339).map_err(|_|"INVALID_DATE")?;if date.nanosecond()%1_000_000!=0{return Err("SUB_MILLISECOND_PRECISION");}i64::try_from(date.unix_timestamp_nanos()/1_000_000).map_err(|_|"DATE_OUT_OF_RANGE")?},
  "from_unix"=>{if object.len()!=2{return Err("INVALID_INPUT");}let value=input["value"].as_i64().ok_or("INTEGER_REQUIRED")?;match input["unit"].as_str(){Some("seconds")=>value.checked_mul(1000).ok_or("DATE_OUT_OF_RANGE")?,Some("milliseconds")=>value,_=>return Err("UNIT_REQUIRED")}},
  _=>return Err("UNKNOWN_ACTION")
 };
 if !(-62_135_596_800_000..=253_402_300_799_999).contains(&millis){return Err("DATE_OUT_OF_RANGE");}
 let date=OffsetDateTime::from_unix_timestamp_nanos(i128::from(millis)*1_000_000).map_err(|_|"DATE_OUT_OF_RANGE")?;
 Ok(json!({"iso_utc":date.format(&Rfc3339).map_err(|_|"INVALID_DATE")?,"unix_seconds":millis.div_euclid(1000),"unix_millis":millis}))
}
fn node(kind:&str,key:&str,props:Value,children:Vec<Value>)->Value{json!({"type":kind,"key":key,"props":props,"children":children})}
pub struct TimestampUtc {iso:String,value:String,unit:String,result:Option<Value>,error:Option<&'static str>,running:bool}
impl Default for TimestampUtc{fn default()->Self{Self{iso:"1970-01-01T00:00:00Z".into(),value:"0".into(),unit:"milliseconds".into(),result:None,error:None,running:false}}}
impl TimestampUtc {
 fn apply(&mut self,action:&str,input:&Value)->Result<Value,&'static str>{match convert(action,input){Ok(v)=>{self.result=Some(v.clone());self.error=None;Ok(v)},Err(e)=>{self.result=None;self.error=Some(e);Err(e)}}}
 fn event(&mut self,p:&Value)->Result<Value,&'static str>{let key=p["node_key"].as_str().unwrap_or("");let kind=p["event_type"].as_str().unwrap_or("");
  if (kind=="input"&&["iso","unix"].contains(&key))||(kind=="change"&&key=="unit"){let value=p["value"].as_str().filter(|s|s.len()<=64).ok_or("INVALID_INPUT")?;match key{"iso"=>self.iso=value.into(),"unix"=>self.value=value.into(),"unit" if ["seconds","milliseconds"].contains(&value)=>self.unit=value.into(),_=>return Err("INVALID_INPUT")};}
  else if kind!="click"||!p["value"].is_null()||!["to_unix","from_unix"].contains(&key){return Err("INVALID_INPUT");}
  if key=="iso"||key=="to_unix"{let iso=self.iso.clone();let _=self.apply("to_unix",&json!({"iso_utc":iso}));}else{let value=self.value.parse::<i64>().ok();let unit=self.unit.clone();let _=self.apply("from_unix",&json!({"value":value,"unit":unit}));}Ok(json!({}))
 }
 fn render(&self)->Value{node("column","timestamp",json!({"gap":12}),vec![
  node("heading","title",json!({"text":"Dates UTC et Unix","level":1}),vec![]),
  node("input","iso",json!({"label":"Date UTC (avec Z)","value":self.iso}),vec![]),node("button","to_unix",json!({"text":"Vers Unix"}),vec![]),
  node("input","unix",json!({"label":"Valeur Unix entière","value":self.value}),vec![]),
  node("select","unit",json!({"label":"Unité Unix","value":self.unit,"options":[{"label":"Secondes","value":"seconds"},{"label":"Millisecondes","value":"milliseconds"}]}),vec![]),node("button","from_unix",json!({"text":"Vers UTC"}),vec![]),
  node("text","result",json!({"text":self.result.as_ref().map(|v|format!("{} · {} secondes · {} millisecondes",v["iso_utc"].as_str().unwrap(),v["unix_seconds"],v["unix_millis"])).unwrap_or_else(||"Saisissez une date ou une valeur Unix.".into())}),vec![]),
  node("alert","status",json!({"text":self.error.unwrap_or("UTC explicite · années 0001 à 9999 · aucune horloge système"),"severity":if self.error.is_some(){"error"}else{"info"}}),vec![])
 ])}
}
impl Tool for TimestampUtc{fn handle(&mut self,r:Request)->Reply{let id=r.request_id;let p=Value::Object(r.payload);let result=match r.kind{RequestKind::Init=>{*self=Self{running:true,..Self::default()};Ok(json!({}))},RequestKind::Shutdown=>{*self=Self::default();Ok(json!({}))},_ if !self.running=>Err("NOT_INITIALIZED"),RequestKind::Render=>Ok(self.render()),RequestKind::ActionInvoke=>self.apply(p["action_id"].as_str().unwrap_or(""),&p["input"]),RequestKind::UiEvent=>self.event(&p),_=>Err("UNSUPPORTED_REQUEST")};match result{Ok(v)=>success(id,v.as_object().unwrap().clone()),Err(e)=>failure(id,e)}}}
kortex_sdk::export_tool!(TimestampUtc);
