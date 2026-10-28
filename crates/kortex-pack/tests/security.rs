use kortex_pack::{Files, archive, wasm_check};
use sha2::{Digest, Sha256};
fn module() -> Vec<u8> {
    wat::parse_str(r#"(module (memory (export "memory") 1 1024) (func (export "kortex_alloc") (param i32) (result i32) i32.const 0) (func (export "kortex_dealloc") (param i32 i32)) (func (export "kortex_dispatch") (param i32 i32) (result i64) i64.const 0))"#).unwrap()
}
fn files() -> Files {
    Files::from([
        (
            "manifest.json".into(),
            include_bytes!("../../../tests/fixtures/calculatrice-v1-manifest.json").to_vec(),
        ),
        (
            "contract.json".into(),
            include_bytes!("../../../tools/calculatrice/contract.json").to_vec(),
        ),
        ("module.wasm".into(), module()),
        ("assets/icon.svg".into(), b"<svg/>".to_vec()),
    ])
}
#[test]
fn hostile_packages_are_rejected_without_instantiation() {
    let mut cases = Vec::new();
    let mut f = files();
    f.insert("extra.txt".into(), vec![1]);
    cases.push(("extra-file", f));
    for (name, field, value) in [
        ("wrong-abi", "abi_version", "2.0"),
        ("identity-conflict", "id", "other-tool"),
    ] {
        let mut f = files();
        let mut m: serde_json::Value = serde_json::from_slice(&f["manifest.json"]).unwrap();
        m[field] = serde_json::json!(value);
        f.insert("manifest.json".into(), serde_json::to_vec(&m).unwrap());
        cases.push((name, f));
    }
    let mut f = files();
    f.insert("assets/icon.svg".into(), vec![0; kortex_pack::MAX_FILE + 1]);
    assert!(!kortex_pack::validate::files(&f).is_ok());
    for (name, wat) in [
        (
            "wasi",
            r#"(module (import "wasi_snapshot_preview1" "fd_write" (func)))"#,
        ),
        ("unbounded", r#"(module (memory (export "memory") 1))"#),
        (
            "shared",
            r#"(module (memory (export "memory") 1 1024 shared))"#,
        ),
        (
            "oversized-memory",
            r#"(module (memory (export "memory") 1 1025))"#,
        ),
        (
            "wrong-signature",
            r#"(module (memory (export "memory") 1 1024) (func (export "kortex_alloc") (param i64)))"#,
        ),
    ] {
        let mut f = files();
        f.insert("module.wasm".into(), wat::parse_str(wat).unwrap());
        cases.push((name, f));
    }
    for (name, f) in cases {
        let bytes = archive::write(&f).unwrap();
        assert!(!kortex_pack::inspect(&bytes).valid, "{name}");
    }
}
#[test]
fn ten_thousand_seeded_zip_and_wasm_mutations_never_panic() {
    let zip = kortex_pack::package(files()).unwrap();
    let wasm = module();
    let mut state = 0x4b4f5254u32;
    let mut digest = Sha256::new();
    let mut rejected = [0usize; 2];
    let mut accepted = [0usize; 2];
    for i in 0..10000 {
        let kind = i % 2;
        let mut b = if kind == 0 { zip.clone() } else { wasm.clone() };
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let at = state as usize % b.len();
        match i % 5 {
            0 => b.truncate(at),
            1 => {
                b[at] ^= 1 << (state % 8);
            }
            2 => {
                b[at] = 0xff;
            }
            3 => {
                b.splice(at..at, [0xff, 0xff, 0xff, 0xff, 0x0f]);
            }
            _ => {
                b.remove(at);
            }
        }
        digest.update((b.len() as u64).to_le_bytes());
        digest.update(&b);
        let result = std::panic::catch_unwind(|| {
            if kind == 0 {
                kortex_pack::inspect(&b).valid
            } else {
                wasm_check::check(&b).is_ok()
            }
        });
        assert!(result.is_ok(), "panic at mutation {i}");
        if result.unwrap() {
            accepted[kind] += 1
        } else {
            rejected[kind] += 1
        }
    }
    let report = serde_json::json!({"seed":"0x4b4f5254","mutations":10000,"zip":5000,"wasm":5000,"rejected":rejected,"structurally_valid_mutations":accepted,"corpus_sha256":format!("{:x}",digest.finalize()),"instantiations":0,"panics":0});
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::create_dir_all(root.join("target")).unwrap();
    std::fs::write(
        root.join("target/pack-security.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    let reference = root.join("tests/fixtures/malformed/corpus.json");
    {
        let expected: serde_json::Value =
            serde_json::from_slice(&std::fs::read(reference).unwrap()).unwrap();
        assert_eq!(report, expected);
    }
    assert!(rejected[0] > 4000 && rejected[1] > 4000);
    println!("{report}");
}
