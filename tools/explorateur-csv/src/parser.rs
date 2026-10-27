//! Strict RFC 4180 style CSV parser with hard bounds; no external crate, no recovery heuristics.
//! Streaming: bytes are consumed chunk by chunk so one guest dispatch never processes the whole file
//! (the host gives each dispatch a fixed instruction budget). Cells live in one arena, not one allocation each.
pub const MAX_ROWS:usize=5000;
pub const MAX_COLUMNS:usize=50;
pub const MAX_BYTES:usize=10*1024*1024;
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Table{pub headers:Vec<String>,arena:String,cells:Vec<(u32,u32)>}
impl Table{
 pub fn columns(&self)->usize{self.headers.len()}
 pub fn rows(&self)->usize{self.cells.len()/self.headers.len().max(1)}
 pub fn cell(&self,row:usize,column:usize)->&str{let (a,b)=self.cells[row*self.headers.len()+column];&self.arena[a as usize..b as usize]}
 pub fn row(&self,row:usize)->Vec<&str>{(0..self.headers.len()).map(|c|self.cell(row,c)).collect()}
}
#[derive(Clone,Copy,PartialEq)]
enum State{Start,Unquoted,Quoted,AfterQuote}
pub struct Parser{state:State,pending_cr:bool,started:bool,total:usize,carry:Vec<u8>,arena:String,cells:Vec<(u32,u32)>,headers:Vec<String>,cell_start:usize,row_len:usize,records:usize}
impl Default for Parser{fn default()->Self{Self::new()}}
impl Parser{
 pub fn new()->Self{Self{state:State::Start,pending_cr:false,started:false,total:0,carry:Vec::new(),arena:String::new(),cells:Vec::new(),headers:Vec::new(),cell_start:0,row_len:0,records:0}}
 /// Number of completed data rows (the header record is not counted).
 pub fn rows(&self)->usize{if self.headers.is_empty(){0}else{self.cells.len()/self.headers.len()}}
 pub fn columns(&self)->usize{self.headers.len()}
 pub fn cell(&self,row:usize,column:usize)->&str{let (a,b)=self.cells[row*self.headers.len()+column];&self.arena[a as usize..b as usize]}
 pub fn headers(&self)->&[String]{&self.headers}
 fn end_field(&mut self)->Result<(),&'static str>{
  if self.row_len>=MAX_COLUMNS{return Err("TOO_LARGE");}
  self.cells.push((self.cell_start as u32,self.arena.len() as u32));self.cell_start=self.arena.len();self.row_len+=1;Ok(())
 }
 fn end_record(&mut self)->Result<(),&'static str>{
  self.end_field()?;self.state=State::Start;
  if self.headers.is_empty(){
   self.headers=self.cells.iter().map(|(a,b)|self.arena[*a as usize..*b as usize].to_string()).collect();self.cells.clear();self.arena.clear();self.cell_start=0;
  }else if self.row_len!=self.headers.len(){return Err("RAGGED_ROW");}
  self.records+=1;if self.records>MAX_ROWS+1{return Err("TOO_LARGE");}
  self.row_len=0;Ok(())
 }
 pub fn feed(&mut self,bytes:&[u8])->Result<(),&'static str>{
  self.total=self.total.checked_add(bytes.len()).filter(|n|*n<=MAX_BYTES).ok_or("TOO_LARGE")?;
  let joined;let data:&[u8]=if self.carry.is_empty(){bytes}else{joined=[self.carry.as_slice(),bytes].concat();&joined};
  let (valid,rest)=match std::str::from_utf8(data){Ok(s)=>(s,&data[data.len()..]),Err(e)=>{if e.error_len().is_some(){return Err("INVALID_UTF8");}let at=e.valid_up_to();(std::str::from_utf8(&data[..at]).map_err(|_|"INVALID_UTF8")?,&data[at..])}};
  let carry=rest.to_vec();
  let valid=if !self.started&&!valid.is_empty(){self.started=true;valid.strip_prefix('\u{feff}').unwrap_or(valid)}else{valid};
  let mut previous=0u8;
  for &b in valid.as_bytes(){if (b<0x20&&!matches!(b,9|10|13))||b==0x7f||(previous==0xc2&&(0x80..=0x9f).contains(&b)){return Err("INVALID_CHARACTER");}previous=b;}
  self.carry=carry;
  let text=valid;let b=text.as_bytes();let n=b.len();let mut i=0;
  while i<n{
   if self.pending_cr{if b[i]!=b'\n'{return Err("INVALID_LINE_ENDING");}self.pending_cr=false;i+=1;self.end_record()?;continue;}
   match self.state{
    State::Start=>match b[i]{b'"'=>{self.state=State::Quoted;i+=1;},b','=>{self.end_field()?;i+=1;},b'\n'=>{i+=1;self.end_record()?;},b'\r'=>{self.pending_cr=true;i+=1;},_=>self.state=State::Unquoted},
    State::Unquoted=>{let mut j=i;while j<n&&!matches!(b[j],b','|b'"'|b'\n'|b'\r'){j+=1;}self.arena.push_str(&text[i..j]);i=j;
     if j<n{match b[j]{b'"'=>return Err("MALFORMED_QUOTE"),b','=>{self.end_field()?;self.state=State::Start;i+=1;},b'\n'=>{i+=1;self.end_record()?;},_=>{self.pending_cr=true;i+=1;}}}}
    State::Quoted=>{let mut j=i;while j<n&&b[j]!=b'"'{j+=1;}self.arena.push_str(&text[i..j]);i=j;if j<n{self.state=State::AfterQuote;i+=1;}}
    State::AfterQuote=>match b[i]{b'"'=>{self.arena.push('"');self.state=State::Quoted;i+=1;},b','=>{self.end_field()?;self.state=State::Start;i+=1;},b'\n'=>{i+=1;self.end_record()?;},b'\r'=>{self.pending_cr=true;i+=1;},_=>return Err("MALFORMED_QUOTE")}
   }
   if self.arena.len()>MAX_BYTES{return Err("TOO_LARGE");}
  }
  Ok(())
 }
 pub fn finish(mut self)->Result<Table,&'static str>{
  if self.pending_cr{return Err("INVALID_LINE_ENDING");}if !self.carry.is_empty(){return Err("INVALID_UTF8");}
  match self.state{State::Quoted=>return Err("UNTERMINATED_QUOTE"),State::Start if self.row_len==0&&self.arena.len()==self.cell_start=>{},_=>self.end_record()?}
  if self.headers.is_empty(){return Err("EMPTY");}
  let table=Table{headers:self.headers,arena:self.arena,cells:self.cells};
  if table.columns()==1&&(0..table.rows()).all(|r|table.cell(r,0).trim().is_empty())&&table.headers[0].trim().is_empty(){return Err("EMPTY");}
  Ok(table)
 }
}
/// Parses a complete in-memory UTF-8 CSV with a mandatory header row. Quotes are only valid around a whole field.
pub fn parse(bytes:&[u8])->Result<Table,&'static str>{let mut parser=Parser::new();parser.feed(bytes)?;parser.finish()}
