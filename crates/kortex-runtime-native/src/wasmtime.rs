use crate::limits::*;
use ::wasmtime::*;
use kortex_abi::{Envelope, Reply, pointer};
use kortex_core::{GuestEngine, RuntimeError, VerifiedTool};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
struct Host {
    limits: StoreLimits,
}
struct Machine {
    store: Store<Host>,
    instance: Instance,
}
fn failure(e: Error) -> RuntimeError {
    if matches!(
        e.downcast_ref::<Trap>(),
        Some(Trap::OutOfFuel | Trap::Interrupt)
    ) {
        RuntimeError::Timeout
    } else {
        RuntimeError::GuestCrash
    }
}
fn machine(
    tool: &VerifiedTool,
    engine: &Engine,
    inbox: Option<(crate::permissions::CapabilityInbox, u64)>,
) -> Result<Machine, RuntimeError> {
    let (manifest, contract, bytes) = crate::package_parts(tool.artifact())?;
    if !bytes.starts_with(b"\0asm\x01\0\0\0") {
        return Err(RuntimeError::UnsupportedAbi);
    }
    if manifest["id"] != tool.manifest().id || manifest["version"] != tool.manifest().version {
        return Err(RuntimeError::IntegrityError);
    }
    if serde_json::to_value(tool.manifest()).map_err(|_| RuntimeError::IntegrityError)? != manifest
        || serde_json::to_value(tool.contract()).map_err(|_| RuntimeError::IntegrityError)?
            != contract
    {
        return Err(RuntimeError::IntegrityError);
    }
    kortex_pack::wasm_check::check(&bytes).map_err(|_| RuntimeError::UnsupportedAbi)?;
    let module = Module::new(engine, &bytes).map_err(failure)?;
    for import in module.imports() {
        if import.module() != "kortex_v1"
            || import.name() != "host_enqueue"
            || !matches!(import.ty(), ExternType::Func(_))
        {
            return Err(RuntimeError::UnsupportedAbi);
        }
    }
    let mem = module
        .exports()
        .find_map(|e| {
            if e.name() == "memory" {
                match e.ty() {
                    ExternType::Memory(m) => Some(m),
                    _ => None,
                }
            } else {
                None
            }
        })
        .ok_or(RuntimeError::UnsupportedAbi)?;
    if mem.is_64()
        || mem.is_shared()
        || mem.maximum().is_none_or(|m| m > 1024)
        || mem.minimum() > 1024
    {
        return Err(RuntimeError::UnsupportedAbi);
    }
    let mut store = Store::new(
        engine,
        Host {
            limits: StoreLimitsBuilder::new()
                .memory_size(MEMORY_BYTES)
                .instances(1)
                .memories(1)
                .tables(1)
                .table_elements(65536)
                .build(),
        },
    );
    store.limiter(|h| &mut h.limits);
    store.set_fuel(FUEL).map_err(failure)?;
    store.set_epoch_deadline(3000);
    let mut linker = Linker::new(engine);
    let declared = tool.manifest().permissions.clone();
    linker
        .func_wrap(
            "kortex_v1",
            "host_enqueue",
            move |mut caller: Caller<'_, Host>, p: i32, n: i32| -> i32 {
                let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                    return -1;
                };
                if pointer::checked_range(p as u32, n as u32, memory.data_size(&caller)).is_err() {
                    return -1;
                }
                let Some((inbox, generation)) = &inbox else {
                    return -1;
                };
                let mut bytes = vec![0; n as u32 as usize];
                if memory.read(&caller, p as u32 as usize, &mut bytes).is_err() {
                    return -1;
                }
                inbox.enqueue(*generation, &bytes, &declared)
            },
        )
        .map_err(failure)?;
    let instance = linker.instantiate(&mut store, &module).map_err(failure)?;
    instance
        .get_typed_func::<i32, i32>(&mut store, "kortex_alloc")
        .map_err(failure)?;
    instance
        .get_typed_func::<(i32, i32), ()>(&mut store, "kortex_dealloc")
        .map_err(failure)?;
    instance
        .get_typed_func::<(i32, i32), i64>(&mut store, "kortex_dispatch")
        .map_err(failure)?;
    Ok(Machine { store, instance })
}
impl Machine {
    fn dispatch(&mut self, r: &Envelope, deadline: u64) -> Result<Reply, RuntimeError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| RuntimeError::Timeout)?
            .as_millis() as u64;
        if deadline <= now {
            return Err(RuntimeError::Timeout);
        }
        self.store
            .set_epoch_deadline(((deadline - now).min(30000) / 10).max(1));
        self.store.set_fuel(FUEL).map_err(failure)?;
        let bytes = r.encode().map_err(|_| RuntimeError::InvalidInput)?;
        let memory = self
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or(RuntimeError::UnsupportedAbi)?;
        let alloc = self
            .instance
            .get_typed_func::<i32, i32>(&mut self.store, "kortex_alloc")
            .map_err(failure)?;
        let free = self
            .instance
            .get_typed_func::<(i32, i32), ()>(&mut self.store, "kortex_dealloc")
            .map_err(failure)?;
        let call = self
            .instance
            .get_typed_func::<(i32, i32), i64>(&mut self.store, "kortex_dispatch")
            .map_err(failure)?;
        let p = alloc
            .call(&mut self.store, bytes.len() as i32)
            .map_err(failure)? as u32;
        pointer::checked_range(p, bytes.len() as u32, memory.data_size(&self.store))
            .map_err(|_| RuntimeError::InvalidInput)?;
        memory
            .write(&mut self.store, p as usize, &bytes)
            .map_err(|_| RuntimeError::InvalidInput)?;
        let packed = call
            .call(&mut self.store, (p as i32, bytes.len() as i32))
            .map_err(failure)?;
        let (out, n) = pointer::unpack(packed);
        pointer::checked_range(out, n, memory.data_size(&self.store))
            .map_err(|_| RuntimeError::InvalidReply)?;
        if (out as u64) < p as u64 + bytes.len() as u64 && (p as u64) < out as u64 + n as u64 {
            return Err(RuntimeError::InvalidReply);
        }
        let mut copy = vec![0; n as usize];
        memory
            .read(&self.store, out as usize, &mut copy)
            .map_err(|_| RuntimeError::InvalidReply)?;
        free.call(&mut self.store, (out as i32, n as i32))
            .map_err(failure)?;
        free.call(&mut self.store, (p as i32, bytes.len() as i32))
            .map_err(failure)?;
        let reply = Reply::decode(&copy).map_err(|_| RuntimeError::InvalidReply)?;
        if reply.request_id != r.request_id {
            return Err(RuntimeError::CorrelationError);
        }
        Ok(reply)
    }
}
enum Command {
    Dispatch(
        Envelope,
        u64,
        futures_channel::oneshot::Sender<Result<Reply, RuntimeError>>,
    ),
}
pub struct NativeInstance {
    tx: mpsc::Sender<Command>,
    stop: Arc<AtomicBool>,
    engine: Engine,
    inbox: Option<(crate::permissions::CapabilityInbox, u64)>,
}
impl Drop for NativeInstance {
    fn drop(&mut self) {
        if let Some((inbox, generation)) = &self.inbox {
            inbox.close(*generation)
        }
        self.stop.store(true, Ordering::SeqCst);
        for _ in 0..3001 {
            self.engine.increment_epoch();
        }
    }
}
pub struct NativeGuestEngine;
async fn receive<T>(rx: futures_channel::oneshot::Receiver<T>) -> Result<T, RuntimeError> {
    rx.await.map_err(|_| RuntimeError::GuestCrash)
}
impl NativeGuestEngine {
    pub async fn instantiate(tool: VerifiedTool) -> Result<NativeInstance, RuntimeError> {
        Self::instantiate_hosted(tool, None).await
    }
    async fn instantiate_hosted(
        tool: VerifiedTool,
        inbox: Option<crate::permissions::CapabilityInbox>,
    ) -> Result<NativeInstance, RuntimeError> {
        let inbox = inbox.map(|v| {
            let generation = v.begin();
            (v, generation)
        });
        let mut cfg = Config::new();
        cfg.consume_fuel(true).epoch_interruption(true);
        let engine = Engine::new(&cfg).map_err(failure)?;
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let (ready, wait) = futures_channel::oneshot::channel();
        let e = engine.clone();
        let flag = stop.clone();
        let timer_engine = engine.clone();
        let timer_stop = stop.clone();
        std::thread::spawn(move || {
            while !timer_stop.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(10));
                timer_engine.increment_epoch();
            }
        });
        let instance = NativeInstance {
            tx,
            stop,
            engine,
            inbox: inbox.clone(),
        };
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(move || {
                let mut state = match machine(&tool, &e, inbox) {
                    Ok(v) => {
                        let _ = ready.send(Ok(()));
                        v
                    }
                    Err(err) => {
                        let _ = ready.send(Err(err));
                        flag.store(true, Ordering::SeqCst);
                        return;
                    }
                };
                while !flag.load(Ordering::SeqCst) {
                    match rx.recv_timeout(Duration::from_millis(10)) {
                        Ok(Command::Dispatch(r, d, reply)) => {
                            let result = state.dispatch(&r, d);
                            let fatal = result.is_err();
                            let _ = reply.send(result);
                            if fatal {
                                break;
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(_) => break,
                    }
                }
                flag.store(true, Ordering::SeqCst);
            })
            .map_err(|_| RuntimeError::GuestCrash)?;
        receive(wait).await??;
        Ok(instance)
    }
}

pub struct HostedNativeEngine {
    inbox: crate::permissions::CapabilityInbox,
}
impl HostedNativeEngine {
    pub fn new(inbox: crate::permissions::CapabilityInbox) -> Self {
        Self { inbox }
    }
}
impl GuestEngine for HostedNativeEngine {
    type Handle = NativeInstance;
    async fn load(&mut self, tool: &VerifiedTool) -> Result<NativeInstance, RuntimeError> {
        NativeGuestEngine::instantiate_hosted(tool.clone(), Some(self.inbox.clone())).await
    }
    async fn dispatch(
        &mut self,
        h: &mut NativeInstance,
        r: &Envelope,
        d: u64,
    ) -> Result<Reply, RuntimeError> {
        NativeGuestEngine.dispatch(h, r, d).await
    }
    async fn unload(&mut self, h: NativeInstance) -> Result<(), RuntimeError> {
        NativeGuestEngine.unload(h).await
    }
}
impl GuestEngine for NativeGuestEngine {
    type Handle = NativeInstance;
    async fn load(&mut self, tool: &VerifiedTool) -> Result<NativeInstance, RuntimeError> {
        Self::instantiate(tool.clone()).await
    }
    async fn dispatch(
        &mut self,
        h: &mut NativeInstance,
        r: &Envelope,
        d: u64,
    ) -> Result<Reply, RuntimeError> {
        let (tx, rx) = futures_channel::oneshot::channel();
        h.tx.send(Command::Dispatch(r.clone(), d, tx))
            .map_err(|_| RuntimeError::GuestCrash)?;
        {
            let mut guard = CancelGuard {
                stop: h.stop.clone(),
                engine: h.engine.clone(),
                armed: true,
            };
            let result = receive(rx).await;
            guard.armed = false;
            result?
        }
    }
    async fn unload(&mut self, h: NativeInstance) -> Result<(), RuntimeError> {
        drop(h);
        Ok(())
    }
}

struct CancelGuard {
    stop: Arc<AtomicBool>,
    engine: Engine,
    armed: bool,
}
impl Drop for CancelGuard {
    fn drop(&mut self) {
        if self.armed {
            self.stop.store(true, Ordering::SeqCst);
            for _ in 0..3001 {
                self.engine.increment_epoch();
            }
        }
    }
}
