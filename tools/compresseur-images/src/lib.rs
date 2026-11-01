mod ui;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::ImageEncoder;
use kortex_sdk::capability::CapabilityOutcome;
use kortex_sdk::{CapabilityRequest, Host, Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

pub const FILE_LIMIT: u64 = 700_000;
pub const CHUNK: usize = 256 * 1024;
const MAX_PIXELS: u64 = 4_000_000;

fn image_format(format: image::ImageFormat) -> Result<&'static str, &'static str> {
    match format {
        image::ImageFormat::Jpeg => Ok("jpeg"),
        image::ImageFormat::Png => Ok("png"),
        image::ImageFormat::WebP => Ok("webp"),
        _ => Err("UNSUPPORTED_FORMAT"),
    }
}

fn extension(format: &str) -> &'static str {
    match format {
        "jpeg" => "jpg",
        "png" => "png",
        "webp" => "webp",
        _ => "img",
    }
}

pub fn compress_image(input: &Value) -> Result<Value, &'static str> {
    if input.as_object().is_none_or(|object| object.len() != 2) {
        return Err("INVALID_INPUT");
    }
    let encoded = input["data_base64"]
        .as_str()
        .filter(|text| !text.is_empty() && text.len() <= 933_336)
        .ok_or("INVALID_INPUT")?;
    let source = STANDARD.decode(encoded).map_err(|_| "INVALID_BASE64")?;
    if source.is_empty() {
        return Err("EMPTY");
    }
    if source.len() as u64 > FILE_LIMIT {
        return Err("TOO_LARGE");
    }
    let quality = input["quality"]
        .as_u64()
        .filter(|quality| (30..=95).contains(quality))
        .ok_or("INVALID_QUALITY")? as u8;
    let reader = image::ImageReader::new(Cursor::new(&source))
        .with_guessed_format()
        .map_err(|_| "INVALID_IMAGE")?;
    let source_format = reader.format().ok_or("UNSUPPORTED_FORMAT")?;
    let source_label = image_format(source_format)?;
    let (width, height) = reader.into_dimensions().map_err(|_| "INVALID_IMAGE")?;
    if width == 0
        || height == 0
        || width > 4096
        || height > 4096
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err("DIMENSIONS_LIMIT");
    }
    let decoded =
        image::load_from_memory_with_format(&source, source_format).map_err(|_| "INVALID_IMAGE")?;
    let rgba = decoded.color().has_alpha().then(|| decoded.to_rgba8());
    let has_transparency = rgba
        .as_ref()
        .is_some_and(|image| image.pixels().any(|pixel| pixel[3] < 255));
    let (output, output_format) = if has_transparency {
        let rgba = rgba.ok_or("ENCODE_FAILED")?;
        let mut output = Vec::new();
        image::codecs::png::PngEncoder::new_with_quality(
            &mut output,
            image::codecs::png::CompressionType::Best,
            image::codecs::png::FilterType::Adaptive,
        )
        .write_image(&rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|_| "ENCODE_FAILED")?;
        (output, "png")
    } else {
        drop(rgba);
        let rgb = decoded.to_rgb8();
        let mut output = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality)
            .write_image(&rgb, width, height, image::ExtendedColorType::Rgb8)
            .map_err(|_| "ENCODE_FAILED")?;
        (output, "jpeg")
    };
    let (bytes, format, reduced) = if output.len() < source.len() {
        (output, output_format, true)
    } else {
        (source.clone(), source_label, false)
    };
    Ok(json!({
        "data_base64": STANDARD.encode(&bytes),
        "format": format,
        "width": width,
        "height": height,
        "original_size": source.len(),
        "compressed_size": bytes.len(),
        "reduced": reduced
    }))
}

#[derive(Clone, Copy)]
enum Kind {
    Pick,
    Read { offset: u64 },
    Save,
}

struct Message {
    generation: u64,
    kind: Kind,
    outcome: CapabilityOutcome,
}

pub struct ImageTool {
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
    buffer: Vec<u8>,
    quality: u8,
    result: Option<(Vec<u8>, String, u32, u32, usize, bool)>,
    error: String,
    notice: String,
}

impl Default for ImageTool {
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
            buffer: Vec::new(),
            quality: 75,
            result: None,
            error: String::new(),
            notice: String::new(),
        }
    }
}

pub(crate) fn explain(code: &str) -> &'static str {
    match code {
        "INVALID_INPUT" => "Les paramètres sont invalides.",
        "INVALID_BASE64" => "Le fichier transmis est invalide.",
        "EMPTY" => "Le fichier est vide.",
        "TOO_LARGE" => "Le fichier dépasse la limite de 700 Ko.",
        "UNSUPPORTED_FORMAT" => "Format non pris en charge. Utilisez JPEG, PNG ou WebP.",
        "INVALID_IMAGE" => "Cette image est illisible ou endommagée.",
        "INVALID_QUALITY" => "La qualité doit être comprise entre 30 et 95.",
        "DIMENSIONS_LIMIT" => "L’image dépasse 4 mégapixels ou 4096 px sur un côté.",
        "ENCODE_FAILED" => "La compression de cette image a échoué.",
        "INVALID_HANDLE" | "INVALID_CHUNK" => "La lecture locale du fichier a échoué.",
        "PERMISSION" => "Autorisation refusée ; le fichier n’a pas été lu ou écrit.",
        "NO_RESULT" => "Compressez d’abord une image.",
        "BUSY" => "Une opération est déjà en cours.",
        _ => "Opération impossible.",
    }
}

impl ImageTool {
    pub fn with_host(host: Host) -> Self {
        Self {
            host,
            ..Self::default()
        }
    }

    fn busy(&self) -> bool {
        self.host.pending_count() != 0
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
            .filter(|sequence| *sequence < 1 << 48)
            .ok_or("REQUEST_LIMIT")?;
        let id = format!("12100000-0000-4000-8000-{:012x}", self.sequence);
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

    fn abort(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.handle.clear();
        self.name.clear();
        self.size = 0;
        self.offset = 0;
        self.buffer.clear();
    }

    fn reset(&mut self) {
        self.abort();
        self.result = None;
        self.error.clear();
        self.notice.clear();
        self.phase = "idle";
    }

    fn start(&mut self) -> Result<(), &'static str> {
        if self.busy() {
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

    fn accept(&mut self, kind: Kind, value: Value) -> Result<(), &'static str> {
        match kind {
            Kind::Pick => {
                if value["cancelled"] == true {
                    self.abort();
                    self.phase = "cancelled";
                    return Ok(());
                }
                self.handle = value["handle"]
                    .as_str()
                    .filter(|handle| {
                        handle.len() == 64 && handle.bytes().all(|c| c.is_ascii_hexdigit())
                    })
                    .ok_or("INVALID_HANDLE")?
                    .into();
                self.size = value["size"]
                    .as_u64()
                    .filter(|size| *size > 0 && *size <= FILE_LIMIT)
                    .ok_or("TOO_LARGE")?;
                self.name = value["name"]
                    .as_str()
                    .unwrap_or("image")
                    .chars()
                    .filter(|character| !character.is_control())
                    .take(256)
                    .collect();
                self.buffer = Vec::with_capacity(self.size as usize);
                self.phase = "reading";
                self.read()
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
                    .filter(|text| text.len() <= 349_528)
                    .ok_or("INVALID_CHUNK")?;
                let bytes = STANDARD.decode(text).map_err(|_| "INVALID_BASE64")?;
                let next = self
                    .offset
                    .checked_add(bytes.len() as u64)
                    .ok_or("TOO_LARGE")?;
                if bytes.is_empty()
                    || bytes.len() > CHUNK
                    || next > self.size
                    || next > FILE_LIMIT
                    || value["eof"].as_bool() != Some(next == self.size)
                {
                    return Err("INVALID_CHUNK");
                }
                self.buffer.extend_from_slice(&bytes);
                self.offset = next;
                if next == self.size {
                    self.phase = "ready";
                    self.error.clear();
                    self.notice.clear();
                    Ok(())
                } else {
                    self.read()
                }
            }
            Kind::Save => {
                self.notice = if value["saved"] == true {
                    "Image enregistrée localement.".into()
                } else {
                    "Enregistrement non confirmé.".into()
                };
                Ok(())
            }
        }
    }

    fn settle(&mut self) {
        let messages =
            std::mem::take(&mut *self.inbox.lock().unwrap_or_else(|error| error.into_inner()));
        for message in messages {
            if message.generation != self.generation {
                continue;
            }
            match message.outcome {
                Ok(data) => {
                    if let Err(error) = self.accept(message.kind, Value::Object(data)) {
                        if matches!(message.kind, Kind::Save) {
                            self.notice = explain(error).into();
                        } else {
                            self.fail(error);
                        }
                    }
                }
                Err(error) if error.code == "CANCELLED" => {
                    self.abort();
                    self.phase = "cancelled";
                }
                Err(error) if matches!(message.kind, Kind::Save) => {
                    self.notice = format!("{} ({})", explain(&error.code), error.code);
                }
                Err(error) => self.fail(&error.code),
            }
        }
    }

    fn fail(&mut self, code: &str) {
        self.abort();
        self.result = None;
        self.phase = "error";
        self.error = code.into();
    }

    fn compress(&mut self) -> Result<(), &'static str> {
        if self.busy() {
            return Err("BUSY");
        }
        if self.phase != "ready" {
            return Err("NO_RESULT");
        }
        self.result = None;
        self.error.clear();
        self.notice.clear();
        let output = compress_image(&json!({
            "data_base64": STANDARD.encode(&self.buffer),
            "quality": self.quality
        }))?;
        let bytes = STANDARD
            .decode(output["data_base64"].as_str().ok_or("ENCODE_FAILED")?)
            .map_err(|_| "ENCODE_FAILED")?;
        self.result = Some((
            bytes,
            output["format"].as_str().ok_or("ENCODE_FAILED")?.into(),
            output["width"].as_u64().ok_or("ENCODE_FAILED")? as u32,
            output["height"].as_u64().ok_or("ENCODE_FAILED")? as u32,
            output["original_size"].as_u64().ok_or("ENCODE_FAILED")? as usize,
            output["reduced"].as_bool().ok_or("ENCODE_FAILED")?,
        ));
        self.phase = "compressed";
        Ok(())
    }

    fn save(&mut self) -> Result<(), &'static str> {
        if self.busy() {
            return Err("BUSY");
        }
        let (bytes, format, ..) = self.result.as_ref().ok_or("NO_RESULT")?;
        let stem = self
            .name
            .rsplit_once('.')
            .map_or(self.name.as_str(), |(stem, _)| stem);
        let name = format!("{stem}-compresse.{}", extension(format));
        self.enqueue(
            Kind::Save,
            "file.save",
            json!({"name":name,"data_base64":STANDARD.encode(bytes)}),
        )
    }

    fn event(&mut self, payload: &Value) -> Result<(), &'static str> {
        let key = payload["node_key"].as_str().unwrap_or("");
        let event = payload["event_type"].as_str().unwrap_or("");
        let result = match (event, key) {
            ("click", "pick") => self.start(),
            ("click", "cancel") => {
                self.abort();
                self.phase = "cancelled";
                Ok(())
            }
            ("click", "compress") => self.compress(),
            ("click", "save") => self.save(),
            ("change", "quality") => {
                let quality = payload["value"]
                    .as_str()
                    .and_then(|value| value.parse::<u8>().ok())
                    .filter(|quality| [45, 60, 75, 85].contains(quality))
                    .ok_or("INVALID_QUALITY")?;
                self.quality = quality;
                self.result = None;
                if self.size > 0 && self.buffer.len() as u64 == self.size {
                    self.phase = "ready";
                }
                self.error.clear();
                self.notice.clear();
                Ok(())
            }
            _ => Err("INVALID_INPUT"),
        };
        if let Err(error) = result {
            self.notice = explain(error).into();
        }
        Ok(())
    }
}

impl Tool for ImageTool {
    fn host(&mut self) -> Option<&mut Host> {
        Some(&mut self.host)
    }

    fn handle(&mut self, request: Request) -> Reply {
        let id = request.request_id;
        let payload = Value::Object(request.payload);
        let result = match request.kind {
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
            RequestKind::ActionInvoke => match payload["action_id"].as_str() {
                Some("compress_image") => compress_image(&payload["input"]),
                _ => Err("UNKNOWN_ACTION"),
            },
            RequestKind::UiEvent => self.event(&payload).map(|_| json!({})),
            _ => Err("UNSUPPORTED_REQUEST"),
        };
        match result {
            Ok(value) => success(id, value.as_object().unwrap().clone()),
            Err(error) => failure(id, error),
        }
    }
}

kortex_sdk::export_tool!(ImageTool);
