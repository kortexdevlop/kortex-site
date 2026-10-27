use crate::{CsvTool,PAGE_SIZE,explain,parser::Table,stats::{self,is_null}};
use serde_json::{Value,json};
fn node(kind:&str,key:&str,props:Value,children:Vec<Value>)->Value{json!({"type":kind,"key":key,"props":props,"children":children})}
fn clip(text:&str,max:usize)->String{if text.chars().count()<=max{text.into()}else{format!("{}…",text.chars().take(max-1).collect::<String>())}}
fn label(name:&str,index:usize)->String{let name=clip(name.trim(),100);if name.is_empty(){format!("Colonne {}",index+1)}else{name}}
impl CsvTool{
 fn rows_matching(&self)->Vec<usize>{let Some(table)=&self.table else{return Vec::new()};let query=self.query.to_lowercase();
  (0..table.rows()).filter(|i|query.is_empty()||table.cell(*i,self.search_col).to_lowercase().contains(&query)).collect()}
 fn preview(&self,table:&Table)->Value{
  let matches=self.rows_matching();let pages=matches.len().div_ceil(PAGE_SIZE).max(1);let page=self.page.min(pages-1);
  let columns:Vec<Value>=table.headers.iter().enumerate().map(|(i,h)|json!({"key":format!("c{i}"),"label":label(h,i)})).collect();
  let rows:Vec<Value>=matches.iter().skip(page*PAGE_SIZE).take(PAGE_SIZE).map(|r|{let mut o=serde_json::Map::new();for i in 0..table.columns(){o.insert(format!("c{i}"),Value::String(clip(table.cell(*r,i),200)));}Value::Object(o)}).collect();
  let options:Vec<Value>=table.headers.iter().enumerate().map(|(i,h)|json!({"label":label(h,i),"value":i.to_string()})).collect();
  node("card","apercu",json!({"title":"Aperçu"}),vec![
   node("select","search_col",json!({"label":"Colonne à rechercher","value":self.search_col.to_string(),"options":options}),vec![]),
   node("input","search",json!({"label":"Rechercher dans la colonne","value":self.query,"input_type":"search"}),vec![]),
   node("text","range",json!({"text":format!("{} ligne(s) sur {} · page {} / {}",matches.len(),table.rows(),page+1,pages)}),vec![]),
   node("table","rows",json!({"columns":columns,"rows":rows}),vec![]),
   node("row","pager",json!({"gap":8}),vec![node("button","prev",json!({"text":"Page précédente","disabled":page==0}),vec![]),node("button","next",json!({"text":"Page suivante","disabled":page+1>=pages}),vec![])])])
 }
 fn stats_tab(&self)->Value{
  let columns=json!([{"key":"name","label":"Colonne"},{"key":"kind","label":"Type"},{"key":"count","label":"Valeurs"},{"key":"nulls","label":"Vides"},{"key":"min","label":"Min"},{"key":"max","label":"Max"},{"key":"mean","label":"Moyenne"}]);
  let rows:Vec<Value>=self.stats.iter().enumerate().map(|(i,s)|json!({"name":label(&s.name,i),"kind":if s.kind==stats::Kind::Number{"Nombre"}else{"Texte"},"count":s.count,"nulls":s.nulls,"min":s.min,"max":s.max,"mean":s.mean})).collect();
  node("card","stats",json!({"title":"Statistiques"}),vec![node("text","policy",json!({"text":"Les cellules vides ne sont pas comptées. Une colonne est numérique si toutes ses valeurs non vides le sont."}),vec![]),node("table","stats_table",json!({"columns":columns,"rows":rows}),vec![])])
 }
 fn chart_tab(&self,table:&Table)->Value{
  let numeric:Vec<usize>=self.stats.iter().enumerate().filter(|(_,s)|s.kind==stats::Kind::Number).map(|(i,_)|i).collect();
  if numeric.is_empty(){return node("card","graphique",json!({"title":"Graphique"}),vec![node("text","chart_none",json!({"text":"Aucune colonne numérique à représenter."}),vec![])]);}
  let col=if numeric.contains(&self.chart_col){self.chart_col}else{numeric[0]};
  let mut labels=Vec::new();let mut values=Vec::new();let mut unsafe_value=false;
  for i in 0..table.rows(){if labels.len()>=500{break;}let cell=table.cell(i,col);if is_null(cell){continue;}
   match stats::parse_number(cell){Ok(Some(v)) if v.abs()<=9007199254740991.0=>{labels.push((i+1).to_string());values.push(v);},_=>{unsafe_value=true;break;}}}
  let options:Vec<Value>=numeric.iter().map(|i|json!({"label":label(&table.headers[*i],*i),"value":i.to_string()})).collect();
  let chart=if unsafe_value{node("alert","chart_limit",json!({"text":"Des valeurs dépassent la plage affichable ; consultez l’onglet Statistiques.","severity":"warning"}),vec![])}
   else{node("svg-chart","chart",json!({"kind":self.chart_kind,"labels":labels,"series":[{"label":label(&table.headers[col],col),"values":values}]}),vec![])};
  node("card","graphique",json!({"title":"Graphique"}),vec![
   node("select","chart_col",json!({"label":"Colonne numérique","value":col.to_string(),"options":options}),vec![]),
   node("select","chart_kind",json!({"label":"Type de graphique","value":self.chart_kind,"options":[{"label":"Barres","value":"bar"},{"label":"Courbe","value":"line"}]}),vec![]),
   node("text","chart_note",json!({"text":"500 premières valeurs non vides au maximum, numérotées par ligne du fichier."}),vec![]),chart])
 }
 pub(crate) fn render(&self)->Value{
  let busy=self.host.pending_count()>0;
  let message=match self.phase{"picking"=>"Choisissez un fichier CSV après autorisation.".to_string(),"reading"=>format!("Lecture locale : {} / {} octets",self.offset,self.size),
   "done"=>"Analyse terminée. Aucun fichier envoyé.".into(),"cancelled"=>"Analyse annulée. Aucune donnée conservée.".into(),"error"=>format!("{} ({})",explain(&self.error),self.error),_=>"Sélectionnez un fichier CSV UTF-8 de 10 Mio maximum (5 000 lignes, 50 colonnes).".into()};
  let mut children=vec![
   node("heading","title",json!({"text":"Explorateur CSV","level":1}),vec![]),
   node("row","actions",json!({"gap":8}),vec![node("button","pick",json!({"text":"Choisir un fichier CSV","disabled":busy||self.phase=="reading"}),vec![]),node("button","cancel",json!({"text":"Annuler","disabled":!["picking","reading"].contains(&self.phase)}),vec![])]),
   node("text","filename",json!({"text":if self.name.is_empty(){"Aucun fichier sélectionné"}else{&self.name}}),vec![]),
   node("progress","progress",json!({"label":"Progression de la lecture","value":if self.phase=="done"{100.0}else if self.size==0{0.0}else{self.offset as f64/self.size as f64*100.0}}),vec![]),
   node("alert","status",json!({"text":message,"severity":if self.phase=="error"{"error"}else{"info"}}),vec![])];
  if let Some(table)=&self.table{
   children.push(node("tabs","tabs",json!({"label":"Sections","active_key":self.tab}),vec![self.preview(table),self.stats_tab(),self.chart_tab(table)]));
   children.push(node("row","report",json!({"gap":8}),vec![node("button","export",json!({"text":"Exporter le rapport","disabled":busy}),vec![]),node("button","prefs_save",json!({"text":"Mémoriser ce fichier","disabled":busy}),vec![])]));
  }
  children.push(node("row","recent_actions",json!({"gap":8}),vec![node("button","prefs_load",json!({"text":"Afficher les fichiers récents","disabled":busy}),vec![])]));
  children.push(node("text","recent",json!({"text":if self.recent.is_empty(){"Aucun fichier récent affiché.".to_string()}else{format!("Récents : {}",self.recent.iter().map(|n|clip(n,80)).collect::<Vec<_>>().join(" · "))}}),vec![]));
  if !self.notice.is_empty(){children.push(node("alert","notice",json!({"text":clip(&self.notice,500),"severity":"info"}),vec![]));}
  node("column","csv-tool",json!({"gap":12}),children)
 }
}
