use base64::{Engine as _, engine::general_purpose::STANDARD};
use kortex_sdk::capability::CapabilityOutcome;
use kortex_sdk::{CapabilityRequest, Host, Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
pub const FILE_LIMIT: u64 = 10 * 1024 * 1024;
pub const CHUNK: usize = 256 * 1024;
pub fn sha256_bytes(input: &Value) -> Result<Value, &'static str> {
    if input.as_object().is_none_or(|o| o.len() != 1) {
        return Err("INVALID_INPUT");
    }
    let text = input["data_base64"]
        .as_str()
        .filter(|s| s.len() <= 1_048_000)
        .ok_or("SIZE_LIMIT")?;
    let bytes = STANDARD.decode(text).map_err(|_| "INVALID_BASE64")?;
    if bytes.len() > 1024 * 1024 {
        return Err("SIZE_LIMIT");
    }
    Ok(json!({"hex":format!("{:x}",Sha256::digest(bytes))}))
}
#[derive(Clone, Copy)]
enum Kind {
    Pick,
    Read { offset: u64 },
}
struct Message {
    generation: u64,
    kind: Kind,
    outcome: CapabilityOutcome,
}
pub struct HashTool {
    host: Host,
    inbox: Arc<Mutex<Vec<Message>>>,
    generation: u64,
    sequence: u64,
    running: bool,
    phase: &'static str,
    handle: String,
    name: String,
    size: u64,
    offset: u64,
    hasher: Option<Sha256>,
    hex: String,
    error: String,
}
impl Default for HashTool {
    fn default() -> Self {
        Self {
            host: Host::default(),
            inbox: Arc::default(),
            generation: 0,
            sequence: 0,
            running: false,
            phase: "idle",
            handle: String::new(),
            name: String::new(),
            size: 0,
            offset: 0,
            hasher: None,
            hex: String::new(),
            error: String::new(),
        }
    }
}
impl HashTool {
    pub fn with_host(host: Host) -> Self {
        Self {
            host,
            ..Self::default()
        }
    }
    fn enqueue(
        &mut self,
        kind: Kind,
        capability: &str,
        payload: Value,
    ) -> Result<(), &'static str> {
        self.sequence = self
            .sequence
            .checked_add(1)
            .filter(|n| *n < 1 << 48)
            .ok_or("REQUEST_LIMIT")?;
        let id = format!("12010000-0000-4000-8000-{:012x}", self.sequence);
        self.host
            .enqueue(CapabilityRequest {
                request_id: id.clone(),
                capability: capability.into(),
                payload: payload.as_object().unwrap().clone(),
            })
            .map_err(|_| "CAPABILITY_DENIED")?;
        let inbox = self.inbox.clone();
        let generation = self.generation;
        self.host
            .on_result(&id, move |outcome| {
                if let Ok(mut queue) = inbox.lock() {
                    queue.push(Message {
                        generation,
                        kind,
                        outcome,
                    });
                }
            })
            .map_err(|_| "CAPABILITY_ERROR")?;
        Ok(())
    }
    fn fail(&mut self, code: &str) {
        self.phase = "error";
        self.error = code.into();
        self.handle.clear();
        self.hasher = None;
        self.hex.clear();
    }
    fn reset(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.handle.clear();
        self.name.clear();
        self.size = 0;
        self.offset = 0;
        self.hex.clear();
        self.error.clear();
        self.hasher = None;
    }
    fn start(&mut self) -> Result<(), &'static str> {
        if self.host.pending_count() != 0 {
            return Err("BUSY");
        }
        self.reset();
        self.phase = "picking";
        self.enqueue(Kind::Pick, "file.pick", json!({}))
    }
    fn read(&mut self) -> Result<(), &'static str> {
        self.enqueue(
            Kind::Read {
                offset: self.offset,
            },
            "file.read",
            json!({"handle":self.handle,"offset":self.offset,"length":CHUNK}),
        )
    }
    fn finish(&mut self) {
        self.hex = format!("{:x}", self.hasher.take().unwrap().finalize());
        self.handle.clear();
        self.phase = "done";
    }
    fn accept(&mut self, kind: Kind, value: Value) -> Result<(), &'static str> {
        match kind {
            Kind::Pick => {
                if value["cancelled"] == true {
                    self.reset();
                    self.phase = "cancelled";
                    return Ok(());
                }
                let handle = value["handle"]
                    .as_str()
                    .filter(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
                    .ok_or("INVALID_HANDLE")?;
                self.size = value["size"]
                    .as_u64()
                    .filter(|s| *s <= FILE_LIMIT)
                    .ok_or("SIZE_LIMIT")?;
                self.handle = handle.into();
                self.name = value["name"]
                    .as_str()
                    .unwrap_or("Fichier local")
                    .chars()
                    .take(256)
                    .collect();
                self.hasher = Some(Sha256::new());
                self.phase = "reading";
                if self.size == 0 {
                    self.finish();
                    Ok(())
                } else {
                    self.read()
                }
            }
            Kind::Read { offset } => {
                if self.phase != "reading"
                    || value["offset"].as_u64() != Some(offset)
                    || offset != self.offset
                {
                    return Err("INVALID_CHUNK");
                }
                let text = value["data_base64"]
                    .as_str()
                    .filter(|s| s.len() <= 349528)
                    .ok_or("INVALID_CHUNK")?;
                let bytes = STANDARD.decode(text).map_err(|_| "INVALID_BASE64")?;
                let next = self
                    .offset
                    .checked_add(bytes.len() as u64)
                    .ok_or("SIZE_LIMIT")?;
                if bytes.is_empty()
                    || bytes.len() > CHUNK
                    || next > self.size
                    || next > FILE_LIMIT
                    || value["eof"].as_bool() != Some(next == self.size)
                {
                    return Err("INVALID_CHUNK");
                }
                self.hasher.as_mut().ok_or("CANCELLED")?.update(bytes);
                self.offset = next;
                if next == self.size {
                    self.finish();
                    Ok(())
                } else {
                    self.read()
                }
            }
        }
    }
    fn settle(&mut self) {
        let messages = std::mem::take(&mut *self.inbox.lock().unwrap_or_else(|e| e.into_inner()));
        for message in messages {
            if message.generation != self.generation {
                continue;
            }
            match message.outcome {
                Ok(data) => {
                    if let Err(e) = self.accept(message.kind, Value::Object(data)) {
                        self.fail(e);
                    }
                }
                Err(e) => {
                    if e.code == "CANCELLED" {
                        self.reset();
                        self.phase = "cancelled";
                    } else {
                        self.fail(&e.code);
                    }
                }
            }
        }
    }
    fn render(&self) -> Value {
        let message = match self.phase {
            "picking" => "Choisissez un fichier après autorisation.".into(),
            "reading" => format!("Lecture locale : {} / {} octets", self.offset, self.size),
            "done" => "Empreinte terminée. Aucun fichier envoyé.".into(),
            "cancelled" => "Calcul annulé. Aucun résultat conservé.".into(),
            "error" => format!("Accès ou lecture impossible : {}", self.error),
            _ => "Sélectionnez un fichier local de 10 Mio maximum.".into(),
        };
        node(
            "column",
            "hash-tool",
            json!({"gap":12}),
            vec![
                node(
                    "heading",
                    "title",
                    json!({"text":"Empreinte SHA-256","level":1}),
                    vec![],
                ),
                node(
                    "button",
                    "pick",
                    json!({"text":"Choisir un fichier","disabled":self.host.pending_count()>0||self.phase=="reading"}),
                    vec![],
                ),
                node(
                    "button",
                    "cancel",
                    json!({"text":"Annuler le calcul","disabled":!["picking","reading"].contains(&self.phase)}),
                    vec![],
                ),
                node(
                    "text",
                    "filename",
                    json!({"text":if self.name.is_empty(){"Aucun fichier sélectionné"}else{&self.name}}),
                    vec![],
                ),
                node(
                    "progress",
                    "progress",
                    json!({"label":"Progression du calcul","value":if self.phase=="done"{100.0}else if self.size==0{0.0}else{self.offset as f64/self.size as f64*100.0}}),
                    vec![],
                ),
                node(
                    "text",
                    "checksum",
                    json!({"text":if self.hex.is_empty(){"Aucune empreinte disponible"}else{&self.hex}}),
                    vec![],
                ),
                node(
                    "alert",
                    "status",
                    json!({"text":message,"severity":if self.phase=="error"{"error"}else{"info"}}),
                    vec![],
                ),
            ],
        )
    }
}
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
impl Tool for HashTool {
    fn host(&mut self) -> Option<&mut Host> {
        Some(&mut self.host)
    }
    fn handle(&mut self, r: Request) -> Reply {
        let id = r.request_id;
        let p = Value::Object(r.payload);
        let result = match r.kind {
            RequestKind::Init => {
                self.reset();
                self.running = true;
                self.phase = "idle";
                Ok(json!({}))
            }
            RequestKind::Shutdown => {
                self.reset();
                self.running = false;
                self.phase = "idle";
                Ok(json!({}))
            }
            _ if !self.running => Err("NOT_INITIALIZED"),
            RequestKind::Render => Ok(self.render()),
            RequestKind::CapabilityResult => {
                self.settle();
                Ok(json!({}))
            }
            RequestKind::ActionInvoke => {
                if p["action_id"] == "sha256_bytes" {
                    sha256_bytes(&p["input"])
                } else {
                    Err("UNKNOWN_ACTION")
                }
            }
            RequestKind::UiEvent => {
                if p["event_type"] != "click" || !p["value"].is_null() {
                    Err("INVALID_INPUT")
                } else {
                    match p["node_key"].as_str() {
                        Some("pick") => {
                            if let Err(e) = self.start() {
                                self.fail(e);
                            }
                            Ok(json!({}))
                        }
                        Some("cancel") => {
                            self.reset();
                            self.phase = "cancelled";
                            Ok(json!({}))
                        }
                        _ => Err("INVALID_INPUT"),
                    }
                }
            }
            _ => Err("UNSUPPORTED_REQUEST"),
        };
        match result {
            Ok(v) => success(id, v.as_object().unwrap().clone()),
            Err(e) => failure(id, e),
        }
    }
}
kortex_sdk::export_tool!(HashTool);
