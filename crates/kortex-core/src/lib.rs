#![allow(async_fn_in_trait)]
//! Portable coordination core. Engines own execution, cancellation and package extraction.
pub mod dispatcher;
pub mod errors;
pub mod lifecycle;
use dispatcher::{Clock, Dispatcher, Queued, bounded};
use ed25519_dalek::{Signature, VerifyingKey};
pub use errors::RuntimeError;
use kortex_abi::{Envelope, MAX_ACTION_MS, MAX_PACKAGE_BYTES, Reply, RequestKind};
use kortex_shared::{Contract, Manifest, validate_contract, validate_manifest};
pub use lifecycle::State;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
pub type InstanceId = u64;
pub struct ArtifactDescriptor {
    pub schema_version: u8,
    pub tool_id: String,
    pub version: String,
    pub abi_version: String,
    pub sha256: String,
    pub signing_key_id: String,
    pub signature: [u8; 64],
}
/// A verified, immutable signed archive. Does not claim ZIP parsing or guest validation.
pub struct VerifiedTool {
    manifest: Manifest,
    contract: Contract,
    artifact: Vec<u8>,
}
impl VerifiedTool {
    pub fn verify(
        manifest: &Value,
        contract: &Value,
        artifact: Vec<u8>,
        proof: &ArtifactDescriptor,
        trusted_keys: &BTreeMap<String, [u8; 32]>,
    ) -> Result<Self, RuntimeError> {
        let manifest = validate_manifest(manifest).map_err(|_| RuntimeError::IntegrityError)?;
        let contract = validate_contract(contract).map_err(|_| RuntimeError::IntegrityError)?;
        if artifact.is_empty()
            || artifact.len() > MAX_PACKAGE_BYTES
            || proof.schema_version != 1
            || proof.tool_id != manifest.id
            || contract.tool_id != manifest.id
            || proof.version != manifest.version
            || proof.abi_version != manifest.abi_version
        {
            return Err(RuntimeError::IntegrityError);
        }
        let digest = format!("{:x}", Sha256::digest(&artifact));
        if digest != proof.sha256 {
            return Err(RuntimeError::IntegrityError);
        }
        let key = trusted_keys
            .get(&proof.signing_key_id)
            .ok_or(RuntimeError::UntrustedKey)?;
        let key = VerifyingKey::from_bytes(key).map_err(|_| RuntimeError::SignatureInvalid)?;
        let text = format!(
            "kortex-artifact-v1\n{}\n{}\n{}\n{}\n",
            proof.tool_id, proof.version, proof.abi_version, proof.sha256
        );
        key.verify_strict(text.as_bytes(), &Signature::from_bytes(&proof.signature))
            .map_err(|_| RuntimeError::SignatureInvalid)?;
        Ok(Self {
            manifest,
            contract,
            artifact,
        })
    }
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn contract(&self) -> &Contract {
        &self.contract
    }
    pub fn artifact(&self) -> &[u8] {
        &self.artifact
    }
}
/// Trusted adapter. A dropped dispatch/load future must stop work; unload consumes all resources.
/// Load must validate/extract the signed archive, match the provided metadata, and verify guest ABI.
/// Handles and partially completed loads must release resources on Drop, including caller cancellation.
/// Poll methods must not block the executor; native fuel/epoch and Worker termination belong to adapters.
pub trait GuestEngine {
    type Handle;
    async fn load(&mut self, tool: &VerifiedTool) -> Result<Self::Handle, RuntimeError>;
    async fn dispatch(
        &mut self,
        handle: &mut Self::Handle,
        request: &Envelope,
        deadline_ms: u64,
    ) -> Result<Reply, RuntimeError>;
    async fn unload(&mut self, handle: Self::Handle) -> Result<(), RuntimeError>;
}
pub struct Instance<H> {
    state: State,
    handle: H,
    tool: VerifiedTool,
    dispatcher: Dispatcher,
}
impl<H> Instance<H> {
    pub fn state(&self) -> State {
        self.state
    }
    pub fn queued(&self) -> usize {
        self.dispatcher.len()
    }
    pub fn tool_id(&self) -> &str {
        &self.tool.manifest.id
    }
}
pub struct Runtime<E: GuestEngine, C: Clock> {
    engine: E,
    clock: C,
    instances: BTreeMap<InstanceId, Instance<E::Handle>>,
    sequence: u64,
}
impl<E: GuestEngine, C: Clock> Runtime<E, C> {
    pub fn new(engine: E, clock: C) -> Self {
        Self {
            engine,
            clock,
            instances: BTreeMap::new(),
            sequence: 0,
        }
    }
    fn next(&mut self) -> Result<u64, RuntimeError> {
        self.sequence = self
            .sequence
            .checked_add(1)
            .filter(|n| *n <= 0xffff_ffff_ffff)
            .ok_or(RuntimeError::CounterExhausted)?;
        Ok(self.sequence)
    }
    fn request(
        &mut self,
        kind: RequestKind,
        payload: Map<String, Value>,
    ) -> Result<Envelope, RuntimeError> {
        let n = self.next()?;
        Ok(Envelope {
            v: 1,
            request_id: format!("00000000-0000-4000-8000-{n:012x}"),
            kind,
            payload,
        })
    }
    pub fn instance(&self, id: InstanceId) -> Option<&Instance<E::Handle>> {
        self.instances.get(&id)
    }
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
    pub async fn load(&mut self, tool: VerifiedTool) -> Result<InstanceId, RuntimeError> {
        if tool.manifest.abi_version != "1.0" {
            return Err(RuntimeError::UnsupportedAbi);
        }
        let id = self.next()?;
        let deadline = self
            .clock
            .now_ms()
            .checked_add(MAX_ACTION_MS as u64)
            .ok_or(RuntimeError::Timeout)?;
        let handle = bounded(self.engine.load(&tool), &self.clock, deadline).await?;
        let mut state = State::Created;
        state.transition(State::Loaded)?;
        self.instances.insert(
            id,
            Instance {
                state,
                handle,
                tool,
                dispatcher: Dispatcher::default(),
            },
        );
        Ok(id)
    }
    async fn exchange(
        &mut self,
        instance: &mut Instance<E::Handle>,
        request: &Envelope,
        timeout_ms: u32,
    ) -> Result<Reply, RuntimeError> {
        if request.encode().is_err() {
            return Err(RuntimeError::InvalidInput);
        }
        let deadline = self
            .clock
            .now_ms()
            .checked_add(timeout_ms as u64)
            .ok_or(RuntimeError::Timeout)?;
        let reply = bounded(
            self.engine
                .dispatch(&mut instance.handle, request, deadline),
            &self.clock,
            deadline,
        )
        .await?;
        if reply.request_id != request.request_id {
            return Err(RuntimeError::CorrelationError);
        }
        if reply.encode().is_err() {
            return Err(RuntimeError::InvalidReply);
        }
        Ok(reply)
    }
    async fn cleanup(
        &mut self,
        mut instance: Instance<E::Handle>,
        state: State,
    ) -> Result<(), RuntimeError> {
        let _ = instance.state.transition(state);
        self.engine.unload(instance.handle).await
    }
    pub async fn initialize(&mut self, id: InstanceId) -> Result<Reply, RuntimeError> {
        if self
            .instances
            .get(&id)
            .ok_or(RuntimeError::UnknownInstance)?
            .state
            != State::Loaded
        {
            return Err(RuntimeError::InvalidState);
        }
        let init = self.request(RequestKind::Init, Map::new())?;
        let render = self.request(RequestKind::Render, Map::new())?;
        let mut instance = self
            .instances
            .remove(&id)
            .ok_or(RuntimeError::UnknownInstance)?;
        let outcome = async {
            let reply = self.exchange(&mut instance, &init, MAX_ACTION_MS).await?;
            if !reply.ok {
                return Err(RuntimeError::GuestError(
                    reply.error.ok_or(RuntimeError::InvalidReply)?,
                ));
            }
            instance.state.transition(State::Initialized)?;
            let reply = self.exchange(&mut instance, &render, MAX_ACTION_MS).await?;
            if !reply.ok {
                return Err(RuntimeError::GuestError(
                    reply.error.clone().ok_or(RuntimeError::InvalidReply)?,
                ));
            }
            instance.state.transition(State::Running)?;
            Ok(reply)
        }
        .await;
        match outcome {
            Ok(reply) => {
                self.instances.insert(id, instance);
                Ok(reply)
            }
            Err(error) => match self.cleanup(instance, State::Error).await {
                Ok(()) => Err(error),
                Err(cleanup) => Err(RuntimeError::CleanupFailure {
                    cause: Box::new(error),
                    cleanup: Box::new(cleanup),
                }),
            },
        }
    }
    pub fn enqueue_action(
        &mut self,
        id: InstanceId,
        action: &str,
        input: Value,
    ) -> Result<String, RuntimeError> {
        let instance = self
            .instances
            .get(&id)
            .ok_or(RuntimeError::UnknownInstance)?;
        if instance.state != State::Running {
            return Err(RuntimeError::InvalidState);
        }
        let action = instance
            .tool
            .contract
            .actions
            .iter()
            .find(|a| a.id == action)
            .ok_or(RuntimeError::InvalidInput)?;
        let schema = jsonschema::validator_for(&action.input_schema)
            .map_err(|_| RuntimeError::InvalidInput)?;
        if !schema.is_valid(&input) {
            return Err(RuntimeError::InvalidInput);
        }
        let name = action.id.clone();
        let timeout = action.timeout_ms;
        let request = self.request(
            RequestKind::ActionInvoke,
            json!({"action_id":name,"input":input})
                .as_object()
                .cloned()
                .ok_or(RuntimeError::InvalidInput)?,
        )?;
        let correlation = request.request_id.clone();
        self.instances
            .get_mut(&id)
            .ok_or(RuntimeError::UnknownInstance)?
            .dispatcher
            .push(Queued {
                request,
                timeout_ms: timeout,
                action: Some(name),
            })?;
        Ok(correlation)
    }
    pub async fn process_next(&mut self, id: InstanceId) -> Result<Option<Reply>, RuntimeError> {
        let mut instance = self
            .instances
            .remove(&id)
            .ok_or(RuntimeError::UnknownInstance)?;
        if instance.state != State::Running {
            self.instances.insert(id, instance);
            return Err(RuntimeError::InvalidState);
        }
        let Some(queued) = instance.dispatcher.pop() else {
            self.instances.insert(id, instance);
            return Ok(None);
        };
        let mut outcome = self
            .exchange(&mut instance, &queued.request, queued.timeout_ms)
            .await;
        if let Ok(reply) = &outcome {
            if reply.ok {
                if let Some(name) = &queued.action {
                    let schema = instance
                        .tool
                        .contract
                        .actions
                        .iter()
                        .find(|a| &a.id == name)
                        .and_then(|a| jsonschema::validator_for(&a.output_schema).ok());
                    if schema.is_none_or(|s| {
                        !s.is_valid(&Value::Object(reply.data.clone().unwrap_or_default()))
                    }) {
                        outcome = Err(RuntimeError::InvalidOutput)
                    }
                }
            }
        }
        match outcome {
            Ok(reply) => {
                self.instances.insert(id, instance);
                Ok(Some(reply))
            }
            Err(error) => match self.cleanup(instance, State::Error).await {
                Ok(()) => Err(error),
                Err(cleanup) => Err(RuntimeError::CleanupFailure {
                    cause: Box::new(error),
                    cleanup: Box::new(cleanup),
                }),
            },
        }
    }
    pub async fn invoke_action(
        &mut self,
        id: InstanceId,
        action: &str,
        input: Value,
    ) -> Result<Reply, RuntimeError> {
        if !self
            .instances
            .get(&id)
            .ok_or(RuntimeError::UnknownInstance)?
            .dispatcher
            .is_empty()
        {
            return Err(RuntimeError::QueueBusy);
        }
        self.enqueue_action(id, action, input)?;
        self.process_next(id)
            .await?
            .ok_or(RuntimeError::InvalidReply)
    }
    pub async fn dispatch_event(
        &mut self,
        id: InstanceId,
        event: Value,
    ) -> Result<Reply, RuntimeError> {
        let schema: Value =
            serde_json::from_str(include_str!("../../../shared/schemas/ui.schema.json"))
                .map_err(|_| RuntimeError::InvalidInput)?;
        if !jsonschema::validator_for(&schema["$defs"]["event"])
            .map_err(|_| RuntimeError::InvalidInput)?
            .is_valid(&event)
        {
            return Err(RuntimeError::InvalidInput);
        }
        let instance = self
            .instances
            .get(&id)
            .ok_or(RuntimeError::UnknownInstance)?;
        if instance.state != State::Running {
            return Err(RuntimeError::InvalidState);
        }
        if !instance.dispatcher.is_empty() {
            return Err(RuntimeError::QueueBusy);
        }
        let request = self.request(
            RequestKind::UiEvent,
            event
                .as_object()
                .cloned()
                .ok_or(RuntimeError::InvalidInput)?,
        )?;
        self.instances
            .get_mut(&id)
            .ok_or(RuntimeError::UnknownInstance)?
            .dispatcher
            .push(Queued {
                request,
                timeout_ms: MAX_ACTION_MS,
                action: None,
            })?;
        self.process_next(id)
            .await?
            .ok_or(RuntimeError::InvalidReply)
    }
    pub async fn shutdown(&mut self, id: InstanceId) -> Result<(), RuntimeError> {
        let request = self.request(RequestKind::Shutdown, Map::new())?;
        let mut instance = self
            .instances
            .remove(&id)
            .ok_or(RuntimeError::UnknownInstance)?;
        let outcome = if matches!(instance.state, State::Running | State::Initialized) {
            self.exchange(&mut instance, &request, MAX_ACTION_MS)
                .await
                .and_then(|reply| {
                    if reply.ok {
                        Ok(())
                    } else {
                        Err(RuntimeError::GuestError(
                            reply.error.ok_or(RuntimeError::InvalidReply)?,
                        ))
                    }
                })
        } else {
            Ok(())
        };
        let cleanup = self.cleanup(instance, State::Stopped).await;
        match (outcome, cleanup) {
            (Err(cause), Err(cleanup)) => Err(RuntimeError::CleanupFailure {
                cause: Box::new(cause),
                cleanup: Box::new(cleanup),
            }),
            (Err(error), _) | (_, Err(error)) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
        }
    }
}
