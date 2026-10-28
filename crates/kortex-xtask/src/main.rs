use kortex_xtask::{Result, read_bounded};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
const HELP: &str = "Kortex tool builder (local; no publication)\n\n  build --tool <id> | --all [--out <directory>] [--json]\n  validate --tool <id> | --all [--json]\n  package --tool <id> | --all [--out <directory>] [--json]\n  sign --artifact <archive.tool> [--out <descriptor.json>] [--json]\n  verify --artifact <descriptor.json> [--archive <archive.tool>]\n         [--trust <signed-keys.json>] [--root <pinned-root.json>]\n         [--min-trust-version <integer>] [--json]\n\nBuild/validate/package accept --source-dir <external-tool-directory>.\nDefault package directory: target/packages. Build --out copies the WASM there.\nSign requires CI=true, KORTEX_SIGNING_KEY_ID and KORTEX_SIGNING_KEY_SEED_BASE64.\nDefault verification anchors are TEST-ONLY; configure --root/--trust for releases.\nVerify checks hash, signed trust snapshot, signature, then package metadata/ABI.\nvalidate checks the previously built module. package builds before packaging.\nExisting packages with different bytes are never overwritten.\n";
fn main() {
    if let Err(error) = run() {
        if std::env::args().any(|a| a == "--json") {
            eprintln!("{}", json!({"valid":false,"error":error}));
        } else {
            eprintln!("{error}");
        }
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{HELP}");
        return Ok(());
    }
    let command = args[0].as_str();
    let mut options = BTreeMap::new();
    let mut i = 1;
    while i < args.len() {
        let k = &args[i];
        if ![
            "--source-dir",
            "--tool",
            "--all",
            "--out",
            "--json",
            "--artifact",
            "--archive",
            "--trust",
            "--root",
            "--min-trust-version",
        ]
        .contains(&k.as_str())
            || options.contains_key(k)
        {
            return Err("INVALID_ARGUMENT".into());
        }
        let value = if ["--all", "--json"].contains(&k.as_str()) {
            String::new()
        } else {
            i += 1;
            args.get(i)
                .filter(|v| !v.starts_with("--"))
                .ok_or("MISSING_ARGUMENT")?
                .clone()
        };
        options.insert(k.clone(), value);
        i += 1;
    }
    let allowed = match command {
        "build" | "package" => vec!["--tool", "--all", "--out", "--json", "--source-dir"],
        "validate" => vec!["--tool", "--all", "--json", "--source-dir"],
        "sign" => vec!["--artifact", "--out", "--json"],
        "verify" => vec![
            "--artifact",
            "--archive",
            "--trust",
            "--root",
            "--min-trust-version",
            "--json",
        ],
        _ => return Err("UNKNOWN_COMMAND".into()),
    };
    if options.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("INVALID_ARGUMENT_FOR_COMMAND".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let output = if command == "sign" || command == "verify" {
        artifact_command(command, &options, &root)?
    } else {
        if options.contains_key("--all") == options.contains_key("--tool") {
            return Err("SELECT_TOOL_OR_ALL".into());
        }
        let mut tools = if let Some(source) = options.get("--source-dir") {
            if options.contains_key("--all") {
                return Err("SOURCE_REQUIRES_ONE_TOOL".into());
            }
            let tool = kortex_xtask::discovery::source(Path::new(source))?;
            BTreeMap::from([(tool.manifest.id.clone(), tool)])
        } else {
            kortex_xtask::discovery::discover(&root.join("tools"))?
        };
        let selected = if let Some(id) = options.get("--tool") {
            vec![tools.remove(id).ok_or("UNKNOWN_TOOL")?]
        } else {
            tools.into_values().collect()
        };
        if selected.is_empty() {
            return Err("NO_TOOLS".into());
        }
        let out = options
            .get("--out")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("target/packages"));
        let mut results = Vec::new();
        for tool in selected {
            match command {
                "build" => {
                    let path = kortex_xtask::build::build(&root, &tool)?;
                    let mut destination = path.clone();
                    if options.contains_key("--out") {
                        std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
                        destination = out.join(format!("{}.wasm", tool.manifest.id));
                        std::fs::write(&destination, read_bounded(&path, kortex_pack::MAX_FILE)?)
                            .map_err(|e| e.to_string())?;
                    }
                    results.push(
                        json!({"tool_id":tool.manifest.id,"module":destination,"valid":true}),
                    );
                }
                "validate" => {
                    let (_, _, path) = kortex_xtask::build::artifact(&root, &tool)?;
                    let files = kortex_xtask::package::collect(&tool, &path)?;
                    results
                        .push(json!({"tool_id":tool.manifest.id,"valid":true,"files":files.len()}));
                }
                "package" => {
                    let (path, report) = kortex_xtask::package::package(&root, &tool, &out)?;
                    let bytes = read_bounded(&path, kortex_pack::MAX_ARCHIVE)?;
                    let files = kortex_pack::archive::read(&bytes).map_err(|e| e.code)?;
                    let manifest: Value = serde_json::from_slice(&files["manifest.json"])
                        .map_err(|e| e.to_string())?;
                    let contract: Value = serde_json::from_slice(&files["contract.json"])
                        .map_err(|e| e.to_string())?;
                    results.push(json!({"tool_id":tool.manifest.id,"archive":path,"sha256":kortex_verify::sha256(&bytes),"validation":report,"manifest":manifest,"contract":contract,"signature_verified":false}));
                }
                _ => unreachable!(),
            }
        }
        json!({"command":command,"results":results})
    };
    if options.contains_key("--json") {
        println!(
            "{}",
            serde_json::to_string(&output).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}
fn artifact_command(
    command: &str,
    options: &BTreeMap<String, String>,
    root: &Path,
) -> Result<serde_json::Value> {
    let path = PathBuf::from(options.get("--artifact").ok_or("MISSING_ARTIFACT")?);
    if command == "sign" {
        if std::env::var("CI").as_deref() != Ok("true") {
            return Err("SIGNING_REQUIRES_CI_SECRET_INJECTION".into());
        }
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        use zeroize::Zeroizing;
        let key_id =
            std::env::var("KORTEX_SIGNING_KEY_ID").map_err(|_| "MISSING_SIGNING_KEY_ID")?;
        let secret = Zeroizing::new(
            std::env::var("KORTEX_SIGNING_KEY_SEED_BASE64")
                .map_err(|_| "MISSING_CI_SIGNING_SECRET")?,
        );
        let decoded = Zeroizing::new(
            STANDARD
                .decode(secret.as_bytes())
                .map_err(|_| "INVALID_CI_SIGNING_SECRET")?,
        );
        let seed = Zeroizing::new(
            <[u8; 32]>::try_from(decoded.as_slice()).map_err(|_| "INVALID_CI_SIGNING_SECRET")?,
        );
        let bytes = read_bounded(&path, kortex_pack::MAX_ARCHIVE)?;
        let files = kortex_pack::archive::read(&bytes).map_err(|e| e.code)?;
        let (m, _) = kortex_pack::validate::files(&files).map_err(|e| e.code)?;
        let proof = kortex_verify::sign_descriptor(
            &bytes,
            &m.id,
            &m.version,
            &m.abi_version,
            &key_id,
            &seed,
        )
        .map_err(|e| e.to_string())?;
        let out = options
            .get("--out")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("{}.json", path.display())));
        let data = serde_json::to_vec_pretty(&proof).map_err(|e| e.to_string())?;
        if out.exists() {
            if read_bounded(&out, 1048576)? != data {
                return Err("DESCRIPTOR_ALREADY_EXISTS".into());
            }
        } else {
            use std::io::Write;
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&out)
                .map_err(|e| e.to_string())?
                .write_all(&data)
                .map_err(|e| e.to_string())?;
        }
        return Ok(json!({"descriptor":out,"proof":proof}));
    }
    let proof = kortex_verify::Descriptor::parse(&read_bounded(&path, 1048576)?)
        .map_err(|e| e.to_string())?;
    let inferred = path.with_extension("");
    let inferred = if inferred.extension().is_some_and(|e| e == "tool") {
        inferred
    } else {
        inferred.with_extension("tool")
    };
    let archive = options
        .get("--archive")
        .map(PathBuf::from)
        .unwrap_or(inferred);
    let bytes = read_bounded(&archive, kortex_pack::MAX_ARCHIVE)?;
    kortex_verify::verify_archive_hash(&bytes, &proof).map_err(|e| e.to_string())?;
    let trust_path = options
        .get("--trust")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("shared/fixtures/trust-test-keys.json"));
    let root_path = options
        .get("--root")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("shared/fixtures/trust-test-root.json"));
    let document = serde_json::from_slice(&read_bounded(&trust_path, 1048576)?)
        .map_err(|_| "INVALID_TRUST_DOCUMENT")?;
    let anchor = serde_json::from_slice(&read_bounded(&root_path, 1048576)?)
        .map_err(|_| "INVALID_ROOT_ANCHOR")?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as u64;
    let minimum = options
        .get("--min-trust-version")
        .map(|v| v.parse::<u64>().map_err(|_| "INVALID_MINIMUM_VERSION"))
        .transpose()?
        .unwrap_or(1);
    let trust = kortex_verify::keys::TrustStore::from_signed(&document, &anchor, minimum, now)
        .map_err(|e| e.to_string())?;
    kortex_verify::verify_descriptor(&bytes, &proof, &trust, now).map_err(|e| e.to_string())?;
    let report = kortex_pack::inspect(&bytes);
    if !report.valid {
        return Err(serde_json::to_string(&report).unwrap());
    }
    let files = kortex_pack::archive::read(&bytes).map_err(|e| e.code)?;
    let (m, _) = kortex_pack::validate::files(&files).map_err(|e| e.code)?;
    if m.id != proof.tool_id || m.version != proof.version || m.abi_version != proof.abi_version {
        return Err("DESCRIPTOR_METADATA_MISMATCH".into());
    }
    Ok(
        json!({"valid":true,"signature_verified":true,"archive":archive,"sha256":proof.sha256,"tool_id":proof.tool_id,"trust_version":trust.version,"trust_profile":if options.contains_key("--root"){"explicit-host-anchor"}else{"test-only-default"},"validation":report}),
    )
}
