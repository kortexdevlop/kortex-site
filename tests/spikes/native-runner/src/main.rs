use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{error::Error, path::PathBuf};
use wasmtime::*;
type Outcome<T> = Result<T, Box<dyn Error>>;
const HASH: &str = "00591ffa2f5faa8398585cd5068058b46efe5409f8ae6011a361844b907793f8";
struct Host {
    limits: StoreLimits,
}
fn dispatch(store: &mut Store<Host>, instance: &Instance, bytes: &[u8]) -> Outcome<Value> {
    if bytes.is_empty() || bytes.len() > 1048576 {
        return Err("message length rejected".into());
    }
    let memory = instance
        .get_memory(&mut *store, "memory")
        .ok_or("missing memory")?;
    let alloc = instance.get_typed_func::<i32, i32>(&mut *store, "kortex_alloc")?;
    let dealloc = instance.get_typed_func::<(i32, i32), ()>(&mut *store, "kortex_dealloc")?;
    let call = instance.get_typed_func::<(i32, i32), i64>(&mut *store, "kortex_dispatch")?;
    let ptr = alloc.call(&mut *store, bytes.len() as i32)? as u32;
    if ptr == 0 {
        return Err("allocation refused".into());
    }
    memory.write(&mut *store, ptr as usize, bytes)?;
    let packed = call.call(&mut *store, (ptr as i32, bytes.len() as i32))? as u64;
    let p = (packed >> 32) as u32;
    let n = packed as u32;
    if n == 0 || n > 1048576 || p.checked_add(n).is_none() {
        return Err("response bounds".into());
    }
    let mut copy = vec![0; n as usize];
    memory.read(&mut *store, p as usize, &mut copy)?;
    dealloc.call(&mut *store, (p as i32, n as i32))?;
    dealloc.call(&mut *store, (ptr as i32, bytes.len() as i32))?;
    Ok(serde_json::from_slice(&copy)?)
}
fn run() -> Outcome<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let wasm=root.join("tests/spikes/core-guest/target/wasm32-unknown-unknown/release/kortex_core_guest_spike.wasm");
    let bytes = std::fs::read(wasm)?;
    let hash = format!("{:x}", Sha256::digest(&bytes));
    if hash != HASH {
        return Err("exact 3.0 WASM hash mismatch: no rebuild allowed".into());
    }
    let mut config = Config::new();
    config.consume_fuel(true);
    let engine = Engine::new(&config)?;
    let module = Module::new(&engine, &bytes)?;
    for import in module.imports() {
        if import.module() != "kortex_v1" || import.name() != "host_enqueue" {
            return Err("unexpected import".into());
        }
    }
    let mut store = Store::new(
        &engine,
        Host {
            limits: StoreLimitsBuilder::new().memory_size(67108864).build(),
        },
    );
    store.limiter(|s| &mut s.limits);
    store.set_fuel(100_000_000)?;
    let mut linker = Linker::new(&engine);
    linker.func_wrap(
        "kortex_v1",
        "host_enqueue",
        |mut caller: Caller<'_, Host>, p: i32, n: i32| -> i32 {
            let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                return -1;
            };
            let (p, n) = (p as u32, n as u32);
            if n == 0
                || n > 1048576
                || p.checked_add(n)
                    .is_none_or(|end| end as usize > memory.data_size(&caller))
            {
                return -1;
            }
            -1
        },
    )?;
    let instance = linker.instantiate(&mut store, &module)?;
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or("missing memory")?;
    if memory.ty(&store).is_64() || memory.ty(&store).maximum() != Some(1024) {
        return Err("memory contract mismatch".into());
    }
    instance.get_typed_func::<i32, i32>(&mut store, "kortex_alloc")?;
    instance.get_typed_func::<(i32, i32), ()>(&mut store, "kortex_dealloc")?;
    let dispatch_fn = instance.get_typed_func::<(i32, i32), i64>(&mut store, "kortex_dispatch")?;
    let cases: Vec<Value> = serde_json::from_str(include_str!("../wire-cases.json"))?;
    let mut replies = Vec::new();
    for case in &cases {
        let raw = if let Some(raw) = case["raw"].as_str() {
            raw.as_bytes().to_vec()
        } else {
            serde_json::to_vec(&case["request"])?
        };
        store.set_fuel(100_000_000)?;
        let reply = dispatch(&mut store, &instance, &raw)?;
        if reply["ok"] != case["ok"] {
            return Err(format!("case failed: {}", case["name"]).into());
        }
        if let Some(code) = case.get("code") {
            if &reply["error"]["code"] != code {
                return Err("error code mismatch".into());
            }
        }
        if let Some(result) = case.get("result") {
            if &reply["data"]["result"] != result {
                return Err("action result mismatch".into());
            }
        }
        replies.push(reply);
    }
    for browser in ["chromium", "webkit"] {
        let path = root.join(format!("target/spike-3.1/{browser}.json"));
        let captured: Vec<Value> = serde_json::from_slice(&std::fs::read(path)?)?;
        if captured != replies {
            return Err(format!("browser parity mismatch: {browser}").into());
        }
    }
    // Invalid raw offsets/lengths: guest rejects before dereferencing, then host reclaims error reply.
    for (p, n) in [(-1, 1), (0, 0), (0, 1048577)] {
        store.set_fuel(100_000_000)?;
        let bits = dispatch_fn.call(&mut store, (p, n))? as u64;
        let ptr = (bits >> 32) as u32;
        let len = bits as u32;
        let mut data = vec![0; len as usize];
        memory.read(&store, ptr as usize, &mut data)?;
        let response: Value = serde_json::from_slice(&data)?;
        if response["error"]["code"] != "INVALID_BUFFER" {
            return Err("invalid pointer accepted".into());
        }
        instance
            .get_typed_func::<(i32, i32), ()>(&mut store, "kortex_dealloc")?
            .call(&mut store, (ptr as i32, len as i32))?;
    }
    // Separate adversarial module, explicitly not a rebuilt 3.0 guest.
    let spin = Module::new(&engine, "(module (func (export \"spin\") (loop br 0)))")?;
    let spin_instance = Instance::new(&mut store, &spin, &[])?;
    store.set_fuel(10_000)?;
    let trap = spin_instance
        .get_typed_func::<(), ()>(&mut store, "spin")?
        .call(&mut store, ())
        .expect_err("infinite loop must stop");
    if !matches!(trap.downcast_ref::<Trap>(), Some(Trap::OutOfFuel)) {
        return Err("unexpected cancellation trap".into());
    }
    println!(
        "{}",
        json!({"sha256":hash,"wire_cases":cases.len(),"browsers_compared":["chromium","webkit"],"pointer_cases":3,"cpu_interrupted":true,"memory_bytes":memory.data_size(&store),"wasmtime":"50.0.0"})
    );
    Ok(())
}

fn main() { std::thread::Builder::new().name("wasmtime-spike".into()).stack_size(32*1024*1024).spawn(|| { if let Err(error)=run() { eprintln!("{error}"); std::process::exit(1); } }).expect("native test thread").join().expect("native test thread panicked"); }
