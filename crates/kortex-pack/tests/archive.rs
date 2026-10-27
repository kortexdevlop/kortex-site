use kortex_pack::{Files, archive, inspect, package};
fn module() -> Vec<u8> {
    wat::parse_str(r#"(module (memory (export "memory") 1 1024) (func (export "kortex_alloc") (param i32) (result i32) i32.const 0) (func (export "kortex_dealloc") (param i32 i32)) (func (export "kortex_dispatch") (param i32 i32) (result i64) i64.const 0))"#).unwrap()
}
fn files() -> Files {
    Files::from([
        (
            "manifest.json".into(),
            include_bytes!("../../../tools/calculatrice/manifest.json").to_vec(),
        ),
        (
            "contract.json".into(),
            include_bytes!("../../../tools/calculatrice/contract.json").to_vec(),
        ),
        ("module.wasm".into(), module()),
        ("assets/icon.svg".into(), b"<svg/>".to_vec()),
    ])
}
fn cd(bytes: &[u8]) -> usize {
    u32::from_le_bytes(bytes[bytes.len() - 6..bytes.len() - 2].try_into().unwrap()) as usize
}
#[test]
fn deterministic_metadata_and_canonical_headers() {
    let mut a = files();
    let first = package(a.clone()).unwrap();
    let m: serde_json::Value = serde_json::from_slice(&a["manifest.json"]).unwrap();
    a.insert(
        "manifest.json".into(),
        serde_json::to_vec_pretty(&m).unwrap(),
    );
    let second = package(a).unwrap();
    assert_eq!(first, second);
    assert!(inspect(&first).valid);
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&first)).unwrap();
    let mut names = Vec::new();
    for i in 0..zip.len() {
        let f = zip.by_index(i).unwrap();
        names.push(f.name().to_string());
        assert_eq!(f.last_modified().unwrap(), zip::DateTime::default());
        assert_eq!(f.unix_mode().unwrap() & 0o777, 0o644);
        assert_eq!(f.compression(), zip::CompressionMethod::Stored);
    }
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}
#[test]
fn traversal_duplicate_case_links_crc_and_headers_are_rejected() {
    for name in [
        "../escape",
        "/root",
        "assets/../escape",
        "assets//x",
        "C:/x",
        "assets\\x",
        "assets/CON.txt",
        "assets/x.",
    ] {
        assert!(!archive::safe_name(name), "{name}");
    }
    let raw = archive::write(&Files::from([
        ("a.txt".into(), b"abcd".to_vec()),
        ("b.txt".into(), b"efgh".to_vec()),
    ]))
    .unwrap();
    let central = cd(&raw);
    let second = central + 46 + 5;
    for replacement in [b"a.txt", b"A.txt"] {
        let mut bad;
        let local = u32::from_le_bytes(raw[second + 42..second + 46].try_into().unwrap()) as usize;
        bad = raw.clone();
        bad[local + 30..local + 35].copy_from_slice(replacement);
        bad[second + 46..second + 51].copy_from_slice(replacement);
        assert!(archive::read(&bad).is_err());
    }
    let mut bad = raw.clone();
    bad[central + 38..central + 42].copy_from_slice(&(0xa1ffu32 << 16).to_le_bytes());
    assert!(archive::read(&bad).is_err());
    let mut bad = raw.clone();
    bad[central + 38] |= 0x08;
    assert!(archive::read(&bad).is_err());
    let mut bad = raw.clone();
    bad[35] ^= 1;
    assert!(archive::read(&bad).is_err());
    for index in [
        6,
        8,
        14,
        18,
        22,
        26,
        28,
        central + 8,
        central + 16,
        central + 42,
    ] {
        let mut bad = raw.clone();
        bad[index] ^= 1;
        assert!(archive::read(&bad).is_err(), "{index}");
    }
}
#[test]
fn bombs_oversize_and_unlisted_files_fail_closed() {
    use std::io::Write;
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file(
        "bomb",
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    zip.write_all(&vec![0; 1024 * 1024]).unwrap();
    let bomb = zip.finish().unwrap().into_inner();
    assert_eq!(archive::read(&bomb).unwrap_err().code, "ARCHIVE_LIMIT");
    let mut f = files();
    f.insert("assets/unreferenced.txt".into(), vec![1]);
    assert_eq!(package(f).unwrap_err().code, "UNREFERENCED_FILE");
    let mut f = files();
    f.remove("assets/icon.svg");
    assert_eq!(package(f).unwrap_err().code, "MISSING_FILE");
    let mut f = files();
    f.insert("large".into(), vec![0; kortex_pack::MAX_FILE + 1]);
    assert!(archive::write(&f).is_err());
}
#[test]
fn abi_signatures_imports_memory_and_binary_are_checked() {
    for wat in [
        "(module)",
        "(module (memory (export \"memory\") 1))",
        "(module (import \"wasi_snapshot_preview1\" \"x\" (func)))",
        "(module (memory (export \"memory\") 1 2048))",
    ] {
        assert!(kortex_pack::validate::wasm(&wat::parse_str(wat).unwrap()).is_err());
    }
    assert!(kortex_pack::validate::wasm(b"(module)").is_err());
    assert!(kortex_pack::validate::wasm(&module()).is_ok());
    let bad=wat::parse_str(r#"(module (memory (export "memory") 1 1024) (func (export "kortex_alloc") (param i64) (result i32) i32.const 0) (func (export "kortex_dealloc") (param i32 i32)) (func (export "kortex_dispatch") (param i32 i32) (result i64) i64.const 0))"#).unwrap();
    assert!(kortex_pack::validate::wasm(&bad).is_err());
}
#[test]
fn truncated_and_mutated_headers_never_panic() {
    let b = package(files()).unwrap();
    for n in (0..b.len()).step_by(13) {
        assert!(archive::read(&b[..n]).is_err());
    }
    for n in (0..b.len()).step_by(19) {
        let mut bad = b.clone();
        bad[n] ^= 0xff;
        assert!(std::panic::catch_unwind(|| archive::read(&bad)).is_ok());
    }
}

#[test]
fn bounded_deflate_round_trip_and_metadata_binding() {
    use std::io::Write;
    let mut state = 123456789u32;
    let data: Vec<u8> = (0..4096)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        })
        .collect();
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file(
        "data.bin",
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    zip.write_all(&data).unwrap();
    assert_eq!(
        archive::read(&zip.finish().unwrap().into_inner()).unwrap()["data.bin"],
        data
    );
    let mut f = files();
    let mut c: serde_json::Value = serde_json::from_slice(&f["contract.json"]).unwrap();
    c["tool_id"] = serde_json::json!("other-tool");
    f.insert("contract.json".into(), serde_json::to_vec(&c).unwrap());
    assert_eq!(package(f).unwrap_err().code, "METADATA_MISMATCH");
}
