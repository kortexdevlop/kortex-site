#![allow(dead_code)]
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use kortex_desktop::{
    api::Api,
    config::Config,
    installer::{Monitor, Progress},
    models::{ToolDetail, ToolSummary},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
#[derive(Clone)]
pub struct Release {
    pub detail: ToolDetail,
    pub bytes: Vec<u8>,
}
pub struct Fixture {
    pub temp: tempfile::TempDir,
    pub config: Config,
    pub base: kortex_pack::Files,
    pub signer: SigningKey,
    pub now: u64,
}
impl Fixture {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let now = kortex_runtime_native::permissions::now_ms();
        let signer = SigningKey::from_bytes(&[42; 32]);
        let root = SigningKey::from_bytes(&[43; 32]);
        let anchor = kortex_verify::keys::RootKey {
            id: "lifecycle-test-root".into(),
            public_key_base64: STANDARD.encode(root.verifying_key().as_bytes()),
        };
        let mut keys = kortex_verify::keys::SignedKeys {
            schema_version: 1,
            version: 1,
            issued_at: now - 1000,
            expires: now + 86400000,
            root_key_id: anchor.id.clone(),
            keys: vec![kortex_verify::keys::TrustKey {
                id: "lifecycle-test-release".into(),
                public_key_base64: STANDARD.encode(signer.verifying_key().as_bytes()),
                not_before: now - 1000,
                expires: now + 86400000,
                revoked: false,
            }],
            signature: String::new(),
        };
        keys.signature = STANDARD.encode(root.sign(&keys.signing_text().unwrap()).to_bytes());
        let trust = kortex_runtime_native::HostTrust {
            root: anchor,
            document: keys,
            minimum_version: 1,
        };
        let path = temp.path().join("trust.json");
        std::fs::write(&path, serde_json::to_vec(&trust).unwrap()).unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let report: Value = serde_json::from_slice(
            &std::fs::read(root.join("target/phase12/convertisseur-casse/release-report.json"))
                .unwrap(),
        )
        .unwrap();
        let base = kortex_pack::archive::read(
            &std::fs::read(report["archive"].as_str().unwrap()).unwrap(),
        )
        .unwrap();
        let config = Config {
            api_origin: "https://kortex-api-staging.userpgl.workers.dev".into(),
            channel: "staging".into(),
            data_dir: temp.path().join("app"),
            trust_path: path,
            offline: true,
        };
        Self {
            temp,
            config,
            base,
            signer,
            now,
        }
    }
    pub fn release(&self, version: &str, permissions: Vec<String>, broken: bool) -> Release {
        let mut files = self.base.clone();
        let mut manifest: Value = serde_json::from_slice(&files["manifest.json"]).unwrap();
        manifest["id"] = json!("desktop-lifecycle-test");
        manifest["version"] = json!(version);
        manifest["name"] = json!("Vérification bibliothèque");
        manifest["permissions"] = json!(permissions);
        let mut contract: Value = serde_json::from_slice(&files["contract.json"]).unwrap();
        contract["tool_id"] = manifest["id"].clone();
        files.insert(
            "manifest.json".into(),
            serde_json::to_vec(&manifest).unwrap(),
        );
        files.insert(
            "contract.json".into(),
            serde_json::to_vec(&contract).unwrap(),
        );
        if broken {
            files.insert("module.wasm".into(),wat::parse_str(r#"(module (memory (export "memory") 1 1024) (func (export "kortex_alloc") (param i32) (result i32) i32.const 64) (func (export "kortex_dealloc") (param i32 i32)) (func (export "kortex_dispatch") (param i32 i32) (result i64) i64.const 0))"#).unwrap());
        }
        let bytes = kortex_pack::package(files).unwrap();
        self.signed(bytes, manifest, contract)
    }
    pub fn signed(&self, bytes: Vec<u8>, manifest: Value, contract: Value) -> Release {
        let descriptor = kortex_verify::sign_descriptor(
            &bytes,
            manifest["id"].as_str().unwrap(),
            manifest["version"].as_str().unwrap(),
            "1.0",
            "lifecycle-test-release",
            &self.signer.to_bytes(),
        )
        .unwrap();
        let tool = ToolSummary {
            id: manifest["id"].as_str().unwrap().into(),
            version: manifest["version"].as_str().unwrap().into(),
            name: manifest["name"].as_str().unwrap().into(),
            summary: manifest["summary"].as_str().unwrap().into(),
            category: manifest["category"].as_str().unwrap().into(),
            tags: serde_json::from_value(manifest["tags"].clone()).unwrap(),
            icon_path: "assets/icon.svg".into(),
            abi_version: "1.0".into(),
            sha256: descriptor.sha256.clone(),
            published_at: self.now,
        };
        Release {
            detail: ToolDetail {
                tool,
                manifest: serde_json::from_value(manifest).unwrap(),
                contract: serde_json::from_value(contract).unwrap(),
                descriptor,
            },
            bytes,
        }
    }
}
pub struct State {
    pub current: Release,
    pub archives: BTreeMap<String, Vec<u8>>,
    pub mode: String,
    pub requests: usize,
}
pub struct Server {
    pub api: Api,
    pub state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Server {
    pub fn new(release: Release) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let api = Api::loopback_test(&format!("http://{}", listener.local_addr().unwrap()));
        let state = Arc::new(Mutex::new(State {
            archives: BTreeMap::from([(
                release.detail.tool.version.clone(),
                release.bytes.clone(),
            )]),
            current: release,
            mode: "normal".into(),
            requests: 0,
        }));
        let shared = state.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let exit = stop.clone();
        let thread = thread::spawn(move || {
            while !exit.load(Ordering::SeqCst) {
                let (mut stream, _) = match listener.accept() {
                    Ok(v) => v,
                    Err(_) => {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let mut request = Vec::new();
                let mut b = [0; 4096];
                while request.len() < 16384 {
                    match stream.read(&mut b) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => request.extend_from_slice(&b[..n]),
                    }
                    if request.windows(4).any(|v| v == b"\r\n\r\n") {
                        break;
                    }
                }
                let text = String::from_utf8_lossy(&request);
                let line = text.lines().next().unwrap_or("");
                let parts = line.split_whitespace().collect::<Vec<_>>();
                if parts.len() < 2 {
                    continue;
                }
                let head = parts[0] == "HEAD";
                let path = parts[1];
                let state = shared.lock().unwrap();
                let mode = state.mode.clone();
                let d = state.current.detail.clone();
                let mut bytes = if path.starts_with("/api/v1/downloads/") {
                    state
                        .archives
                        .get(path.rsplit('/').next().unwrap_or(""))
                        .cloned()
                        .unwrap_or_default()
                } else {
                    serde_json::to_vec(&d).unwrap()
                };
                drop(state);
                shared.lock().unwrap().requests += 1;
                let mut status = 200;
                let mut extra = String::new();
                let mut len = bytes.len();
                if path.starts_with("/api/v1/downloads/") {
                    match mode.as_str() {
                        "truncated" => bytes.truncate(bytes.len() / 2),
                        "tampered" => {
                            if !bytes.is_empty() {
                                bytes[0] ^= 1
                            }
                        }
                        "oversized" => len = 26 * 1024 * 1024,
                        "redirect" => {
                            status = 302;
                            extra = "Location: https://example.invalid/forbidden\r\n".into()
                        }
                        "revoked" => status = 410,
                        _ => {}
                    }
                    if bytes.is_empty() {
                        status = 404
                    }
                }
                let header = format!(
                    "HTTP/1.1 {status} OK\r\nContent-Length: {len}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n{extra}\r\n"
                );
                let _ = stream.write_all(header.as_bytes());
                if !head {
                    if mode == "slow" {
                        for chunk in bytes.chunks(64) {
                            if stream.write_all(chunk).is_err() {
                                break;
                            }
                            thread::sleep(Duration::from_millis(3));
                        }
                    } else {
                        let _ = stream.write_all(&bytes);
                    }
                }
            }
        });
        Self {
            api,
            state,
            stop,
            thread: Some(thread),
        }
    }
    pub fn publish(&self, release: Release) {
        let mut s = self.state.lock().unwrap();
        s.archives
            .insert(release.detail.tool.version.clone(), release.bytes.clone());
        s.current = release;
        s.mode = "normal".into();
    }
    pub fn mode(&self, mode: &str) {
        self.state.lock().unwrap().mode = mode.into();
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            t.join().unwrap()
        }
    }
}
pub fn monitor() -> Monitor {
    Arc::new(Mutex::new(Progress::default()))
}
pub fn install(f: &Fixture, s: &Server, r: &Release) -> kortex_desktop::library::Installed {
    kortex_desktop::installer::install(
        &f.config,
        &s.api,
        &r.detail,
        &r.detail.manifest.permissions,
        &AtomicBool::new(false),
        &monitor(),
        true,
    )
    .unwrap()
}
