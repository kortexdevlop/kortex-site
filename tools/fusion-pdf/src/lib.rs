pub mod pdf;
mod ui;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use kortex_sdk::capability::CapabilityOutcome;
use kortex_sdk::{CapabilityRequest, Host, Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
/// A host message carries at most 1 MiB, so a saved file is bounded once base64-encoded.
pub const FILE_LIMIT: u64 = 700_000;
pub const CHUNK: usize = 256 * 1024;
const SAVE_NAME: &str = "fusion.pdf";
fn decode_input(input: &Value, key: &str) -> Result<Vec<u8>, &'static str> {
    let bytes = STANDARD
        .decode(input[key].as_str().ok_or("INVALID_INPUT")?)
        .map_err(|_| "INVALID_BASE64")?;
    if bytes.len() as u64 > FILE_LIMIT {
        return Err("TOO_LARGE");
    }
    Ok(bytes)
}
/// Headless action: the UI calls the very same merge code.
pub fn merge_pdfs(input: &Value) -> Result<Value, &'static str> {
    if input.as_object().is_none_or(|o| o.len() != 2) {
        return Err("INVALID_INPUT");
    }
    let first = decode_input(input, "first_base64")?;
    let second = decode_input(input, "second_base64")?;
    let (bytes, pages) = pdf::merge(&first, &second, FILE_LIMIT as usize)?;
    Ok(json!({"pdf_base64":STANDARD.encode(bytes),"pages":pages}))
}
#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Pick { slot: usize },
    Read { slot: usize, offset: u64 },
    Save,
}
struct Message {
    generation: u64,
    kind: Kind,
    outcome: CapabilityOutcome,
}
#[derive(Default, Clone)]
pub(crate) struct Slot {
    pub name: String,
    pub data: Vec<u8>,
    pub pages: usize,
}
pub struct PdfTool {
    pub(crate) host: Host,
    inbox: Arc<Mutex<Vec<Message>>>,
    generation: u64,
    sequence: u64,
    running: bool,
    pub(crate) phase: &'static str,
    pub(crate) slots: [Option<Slot>; 2],
    handle: String,
    name: String,
    pub(crate) size: u64,
    pub(crate) offset: u64,
    buffer: Vec<u8>,
    pub(crate) merged: Option<(Vec<u8>, usize)>,
    pub(crate) error: String,
    pub(crate) notice: String,
}
impl Default for PdfTool {
    fn default() -> Self {
        Self {
            host: Host::default(),
            inbox: Arc::default(),
            generation: 0,
            sequence: 0,
            running: false,
            phase: "idle",
            slots: [None, None],
            handle: String::new(),
            name: String::new(),
            size: 0,
            offset: 0,
            buffer: Vec::new(),
            merged: None,
            error: String::new(),
            notice: String::new(),
        }
    }
}
pub(crate) fn explain(code: &str) -> &'static str {
    match code {
        "INVALID_PDF" => "Ce fichier n’est pas un PDF lisible.",
        "ENCRYPTED" => "Ce PDF est protégé par mot de passe : la fusion est impossible.",
        "UNSUPPORTED_PDF" => "Ce PDF utilise une structure que l’outil ne prend pas en charge.",
        "NO_PAGES" => "Un des PDF ne contient aucune page.",
        "TOO_LARGE" => "Un fichier ou le PDF fusionné dépasse 700 Ko.",
        "EMPTY" => "Le fichier est vide.",
        "PERMISSION" => "Autorisation refusée ; aucun octet n’a été lu ou écrit.",
        "NO_DATA" => "Choisissez d’abord les deux PDF.",
        "NO_RESULT" => "Fusionnez d’abord les deux PDF.",
        "BUSY" => "Une opération est déjà en cours.",
        _ => "Opération impossible.",
    }
}
impl PdfTool {
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
        let id = format!("12040000-0000-4000-8000-{:012x}", self.sequence);
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
        self.drop_loading();
    }
    fn drop_loading(&mut self) {
        self.handle.clear();
        self.name.clear();
        self.buffer = Vec::new();
        self.offset = 0;
        self.size = 0;
    }
    /// Abandon whatever is in flight; late host results are ignored.
    fn abort(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.drop_loading();
    }
    fn reset(&mut self) {
        self.abort();
        self.slots = [None, None];
        self.merged = None;
        self.error.clear();
        self.notice.clear();
        self.phase = "idle";
    }
    fn busy(&self) -> bool {
        self.host.pending_count() != 0
    }
    fn start(&mut self, slot: usize) -> Result<(), &'static str> {
        if self.busy() {
            return Err("BUSY");
        }
        self.abort();
        self.slots[slot] = None;
        self.merged = None;
        self.error.clear();
        self.notice.clear();
        self.phase = "picking";
        self.enqueue(Kind::Pick { slot }, "file.pick", json!({}))
    }
    fn read(&mut self, slot: usize) -> Result<(), &'static str> {
        self.enqueue(
            Kind::Read {
                slot,
                offset: self.offset,
            },
            "file.read",
            json!({"handle":self.handle,"offset":self.offset,"length":CHUNK}),
        )
    }
    fn finish(&mut self, slot: usize) {
        let data = std::mem::take(&mut self.buffer);
        let name = std::mem::take(&mut self.name);
        match pdf::page_count(&data) {
            Ok(pages) => {
                self.slots[slot] = Some(Slot { name, data, pages });
                self.drop_loading();
                self.phase = "idle";
            }
            Err(e) => self.fail(e),
        }
    }
    fn accept(&mut self, kind: Kind, value: Value) -> Result<(), &'static str> {
        match kind {
            Kind::Pick { slot } => {
                if value["cancelled"] == true {
                    self.abort();
                    self.phase = "cancelled";
                    return Ok(());
                }
                let handle = value["handle"]
                    .as_str()
                    .filter(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
                    .ok_or("INVALID_HANDLE")?;
                let size = value["size"]
                    .as_u64()
                    .filter(|s| *s <= FILE_LIMIT)
                    .ok_or("TOO_LARGE")?;
                if size == 0 {
                    return Err("EMPTY");
                }
                self.size = size;
                self.handle = handle.into();
                self.offset = 0;
                self.buffer = Vec::with_capacity(size as usize);
                self.name = value["name"]
                    .as_str()
                    .unwrap_or("Fichier local")
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(256)
                    .collect();
                self.phase = "reading";
                self.read(slot)
            }
            Kind::Read { slot, offset } => {
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
                    .ok_or("TOO_LARGE")?;
                if bytes.is_empty()
                    || bytes.len() > CHUNK
                    || next > self.size
                    || value["eof"].as_bool() != Some(next == self.size)
                {
                    return Err("INVALID_CHUNK");
                }
                self.buffer.extend_from_slice(&bytes);
                self.offset = next;
                if next == self.size {
                    self.finish(slot);
                    Ok(())
                } else {
                    self.read(slot)
                }
            }
            Kind::Save => {
                self.notice = if value["saved"] == true {
                    "PDF fusionné enregistré localement.".into()
                } else {
                    "Enregistrement non confirmé.".into()
                };
                Ok(())
            }
        }
    }
    fn settle(&mut self) {
        let messages = std::mem::take(&mut *self.inbox.lock().unwrap_or_else(|e| e.into_inner()));
        for message in messages {
            if message.generation != self.generation {
                continue;
            }
            let saving = matches!(message.kind, Kind::Save);
            match message.outcome {
                Ok(data) => {
                    if let Err(e) = self.accept(message.kind, Value::Object(data)) {
                        if saving {
                            self.notice = explain(e).into();
                        } else {
                            self.fail(e);
                        }
                    }
                }
                Err(e) if e.code == "CANCELLED" => {
                    self.abort();
                    self.phase = "cancelled";
                }
                Err(e) if saving => self.notice = format!("{} ({})", explain(&e.code), e.code),
                Err(e) => self.fail(&e.code),
            }
        }
    }
    fn merge(&mut self) -> Result<(), &'static str> {
        if self.busy() {
            return Err("BUSY");
        }
        let [Some(first), Some(second)] = &self.slots else {
            return Err("NO_DATA");
        };
        let result = pdf::merge(&first.data, &second.data, FILE_LIMIT as usize);
        self.error.clear();
        self.notice.clear();
        match result {
            Ok(result) => {
                self.phase = "merged";
                self.merged = Some(result);
            }
            Err(e) => {
                self.merged = None;
                self.fail(e);
            }
        }
        Ok(())
    }
    fn save(&mut self) -> Result<(), &'static str> {
        if self.busy() {
            return Err("BUSY");
        }
        let (bytes, _) = self.merged.as_ref().ok_or("NO_RESULT")?;
        let payload = json!({"name":SAVE_NAME,"data_base64":STANDARD.encode(bytes)});
        self.notice.clear();
        self.enqueue(Kind::Save, "file.save", payload)
    }
    fn swap(&mut self) -> Result<(), &'static str> {
        if self.busy() {
            return Err("BUSY");
        }
        self.slots.swap(0, 1);
        self.merged = None;
        self.error.clear();
        self.notice.clear();
        self.phase = "idle";
        Ok(())
    }
    fn event(&mut self, p: &Value) -> Result<(), &'static str> {
        let key = p["node_key"].as_str().unwrap_or("");
        let kind = p["event_type"].as_str().unwrap_or("");
        let result = match (kind, key) {
            ("click", "pick_first") => self.start(0),
            ("click", "pick_second") => self.start(1),
            ("click", "cancel") => {
                self.abort();
                self.phase = "cancelled";
                Ok(())
            }
            ("click", "reset") => {
                self.reset();
                Ok(())
            }
            ("click", "swap") => self.swap(),
            ("click", "merge") => self.merge(),
            ("click", "save") => self.save(),
            _ => return Err("INVALID_INPUT"),
        };
        if let Err(e) = result {
            self.notice = explain(e).into();
        }
        Ok(())
    }
}
impl Tool for PdfTool {
    fn host(&mut self) -> Option<&mut Host> {
        Some(&mut self.host)
    }
    fn handle(&mut self, r: Request) -> Reply {
        let id = r.request_id;
        let p = Value::Object(r.payload);
        let result = match r.kind {
            RequestKind::Init => {
                let host = std::mem::take(&mut self.host);
                *self = Self {
                    host,
                    running: true,
                    ..Self::default()
                };
                Ok(json!({}))
            }
            RequestKind::Shutdown => {
                self.reset();
                self.running = false;
                Ok(json!({}))
            }
            _ if !self.running => Err("NOT_INITIALIZED"),
            RequestKind::Render => Ok(self.render()),
            RequestKind::CapabilityResult => {
                self.settle();
                Ok(json!({}))
            }
            RequestKind::ActionInvoke => match p["action_id"].as_str() {
                Some("merge_pdfs") => merge_pdfs(&p["input"]),
                _ => Err("UNKNOWN_ACTION"),
            },
            RequestKind::UiEvent => self.event(&p).map(|_| json!({})),
            _ => Err("UNSUPPORTED_REQUEST"),
        };
        match result {
            Ok(v) => success(id, v.as_object().unwrap().clone()),
            Err(e) => failure(id, e),
        }
    }
}
kortex_sdk::export_tool!(PdfTool);
