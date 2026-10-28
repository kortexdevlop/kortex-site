use crate::{
    ActionSpec, Compiled, Error, MAX_BYTES, Node, Registry, mapper,
    permissions::{self, Consent, Scope},
};
use kortex_capabilities::policy::Policy;
use kortex_core::dispatcher::Clock;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    future::Future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::{Poll, Waker},
};
#[derive(Clone, Default)]
pub struct Cancellation(Arc<CancelState>);
#[derive(Default)]
struct CancelState {
    cancelled: AtomicBool,
    waker: Mutex<Option<Waker>>,
}
impl Cancellation {
    pub fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::SeqCst);
        if let Some(w) = self.0.waker.lock().unwrap().take() {
            w.wake()
        }
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled.load(Ordering::SeqCst)
    }
    fn poll(&self, cx: &mut std::task::Context<'_>) -> Poll<()> {
        if self.is_cancelled() {
            return Poll::Ready(());
        }
        *self.0.waker.lock().unwrap() = Some(cx.waker().clone());
        if self.is_cancelled() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
#[derive(Clone, Copy)]
pub struct Options {
    pub max_retries: u8,
    pub max_time_ms: u64,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            max_retries: 0,
            max_time_ms: 300000,
        }
    }
}
#[derive(Debug)]
pub struct InvokeError {
    pub retryable: bool,
}
/// Only trusted adapters implement this port. Registry comes from verified loaded packages;
/// invoke must use Runtime::invoke_action and its normal capability gateway, never a second executor.
pub trait Host {
    fn registry(&self) -> &Registry;
    fn policy(&self) -> &Policy;
    fn scopes(&self, node: &Node, spec: &ActionSpec, input: &Value) -> Result<Vec<Scope>, Error>;
    async fn invoke(
        &mut self,
        node: &Node,
        spec: &ActionSpec,
        input: Value,
        deadline_ms: u64,
    ) -> Result<Value, InvokeError>;
}
#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub node_id: String,
    pub status: Error,
    pub attempt: u8,
    pub at_ms: u64,
    pub may_have_effects: bool,
}
pub struct Run {
    plan: Compiled,
    options: Options,
    id: u64,
    index: usize,
    started: Option<u64>,
    revision: Option<u64>,
    terminal: Option<Error>,
    outputs: BTreeMap<String, Value>,
    bytes: usize,
    journal: Vec<Event>,
}
static NEXT_RUN: AtomicU64 = AtomicU64::new(1);
impl Run {
    pub fn new(plan: Compiled, options: Options) -> Result<Self, Error> {
        if options.max_retries > 2 || options.max_time_ms == 0 || options.max_time_ms > 300000 {
            return Err(Error::Limit);
        }
        if options.max_retries > 0 && plan.specs.iter().any(|s| !s.action.idempotent) {
            return Err(Error::RetryForbidden);
        }
        let id = NEXT_RUN
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_add(1))
            .map_err(|_| Error::Limit)?;
        Ok(Self {
            plan,
            options,
            id,
            index: 0,
            started: None,
            revision: None,
            terminal: None,
            outputs: BTreeMap::new(),
            bytes: 0,
            journal: Vec::new(),
        })
    }
    pub fn plan(&self) -> &Compiled {
        &self.plan
    }
    pub fn consent(&self, now_ms: u64, allow_pure: bool) -> Consent {
        Consent::new(
            self.id,
            self.options.max_retries,
            self.plan.fingerprint.clone(),
            now_ms.saturating_add(300000),
            allow_pure,
        )
    }
    pub fn outputs(&self) -> &BTreeMap<String, Value> {
        &self.outputs
    }
    pub fn journal(&self) -> &[Event] {
        &self.journal
    }
    pub fn pending_node(&self) -> Option<&Node> {
        if self.terminal.is_some() {
            return None;
        }
        self.plan
            .order
            .get(self.index)
            .map(|i| &self.plan.workflow.nodes[*i])
    }
    fn waiting(&mut self, node: &Node, now: u64) {
        if !self
            .journal
            .last()
            .is_some_and(|e| e.node_id == node.id && e.status == Error::AwaitingConsent)
        {
            self.journal.push(Event {
                node_id: node.id.clone(),
                status: Error::AwaitingConsent,
                attempt: 0,
                at_ms: now,
                may_have_effects: false,
            });
        }
    }
    pub fn pending_input(&self) -> Result<Value, Error> {
        let i = *self.plan.order.get(self.index).ok_or(Error::Completed)?;
        mapper::input(
            &self.plan.workflow.nodes[i],
            &self.plan.specs[i],
            &self.outputs,
        )
    }
    fn stop(&mut self, error: Error, consent: &mut Consent) -> Error {
        self.terminal = Some(error.clone());
        consent.seal();
        error
    }
    fn unchanged(&self, host: &impl Host, i: usize) -> bool {
        host.registry()
            .get(&self.plan.workflow.nodes[i].action_ref)
            .is_some_and(|s| s.fingerprint() == self.plan.specs[i].fingerprint())
    }
    pub async fn advance(
        &mut self,
        host: &mut impl Host,
        consent: &mut Consent,
        clock: &impl Clock,
        cancel: &Cancellation,
    ) -> Error {
        if let Some(status) = &self.terminal {
            return status.clone();
        }
        let start = *self.started.get_or_insert_with(|| clock.now_ms());
        let end = start.saturating_add(self.options.max_time_ms);
        let revision = *self
            .revision
            .get_or_insert_with(|| host.policy().revocation_revision());
        if revision != host.policy().revocation_revision() {
            return self.stop(Error::Cancelled, consent);
        }
        while self.index < self.plan.order.len() {
            if cancel.is_cancelled() {
                return self.stop(Error::Cancelled, consent);
            }
            if clock.now_ms() >= end {
                return self.stop(Error::Timeout, consent);
            }
            let i = self.plan.order[self.index];
            if !self.unchanged(host, i) {
                return self.stop(Error::Changed, consent);
            }
            let node = self.plan.workflow.nodes[i].clone();
            let spec = self.plan.specs[i].clone();
            let sensitive = permissions::sensitive(node.requires_confirmation, &spec);
            if let Err(error) = consent.check(
                self.id,
                self.options.max_retries,
                &node.id,
                sensitive,
                clock.now_ms(),
            ) {
                if error == Error::AwaitingConsent {
                    self.waiting(&node, clock.now_ms());
                    return error;
                }
                return self.stop(error, consent);
            }
            let input = match mapper::input(&node, &spec, &self.outputs) {
                Ok(v) => v,
                Err(e) => return self.stop(e, consent),
            };
            let scopes = match host.scopes(&node, &spec, &input) {
                Ok(v) => v,
                Err(e) => return self.stop(e, consent),
            };
            if let Err(error) =
                permissions::check_policy(host.policy(), &spec, &scopes, clock.now_ms())
            {
                if error == Error::AwaitingConsent {
                    self.waiting(&node, clock.now_ms());
                    return error;
                }
                return self.stop(error, consent);
            }
            consent.consume(&node.id);
            let mut attempt = 0;
            loop {
                if clock.now_ms() >= consent.deadline() {
                    return self.stop(Error::Expired, consent);
                }
                if cancel.is_cancelled() {
                    return self.stop(Error::Cancelled, consent);
                }
                if clock.now_ms() >= end {
                    return self.stop(Error::Timeout, consent);
                }
                if !self.unchanged(host, i) {
                    return self.stop(Error::Changed, consent);
                }
                if let Err(e) =
                    permissions::check_policy(host.policy(), &spec, &scopes, clock.now_ms())
                {
                    return self.stop(e, consent);
                }
                let revision = host.policy().revocation_revision();
                let deadline = clock
                    .now_ms()
                    .saturating_add(spec.action.timeout_ms as u64)
                    .min(end)
                    .min(consent.deadline());
                let outcome = interruptible(
                    host.invoke(&node, &spec, input.clone(), deadline),
                    clock,
                    deadline,
                    cancel,
                )
                .await;
                match outcome {
                    Ok(Ok(value)) => {
                        if !mapper::valid(&spec.action.output_schema, &value) {
                            self.journal.push(Event {
                                node_id: node.id.clone(),
                                status: Error::InvalidOutput,
                                attempt,
                                at_ms: clock.now_ms(),
                                may_have_effects: permissions::sensitive(false, &spec),
                            });
                            return self.stop(Error::InvalidOutput, consent);
                        }
                        let size = serde_json::to_vec(&value).map_or(MAX_BYTES + 1, |b| b.len());
                        if size > MAX_BYTES || self.bytes + size > 4 * MAX_BYTES {
                            self.journal.push(Event {
                                node_id: node.id.clone(),
                                status: Error::Limit,
                                attempt,
                                at_ms: clock.now_ms(),
                                may_have_effects: permissions::sensitive(false, &spec),
                            });
                            return self.stop(Error::Limit, consent);
                        }
                        self.bytes += size;
                        self.outputs.insert(node.id.clone(), value);
                        self.journal.push(Event {
                            node_id: node.id.clone(),
                            status: Error::Completed,
                            attempt,
                            at_ms: clock.now_ms(),
                            may_have_effects: permissions::sensitive(false, &spec),
                        });
                        self.index += 1;
                        if cancel.is_cancelled() || host.policy().revocation_revision() != revision
                        {
                            return self.stop(Error::Cancelled, consent);
                        }
                        if !self.unchanged(host, i) {
                            return self.stop(Error::Changed, consent);
                        }
                        break;
                    }
                    Ok(Err(error)) => {
                        self.journal.push(Event {
                            node_id: node.id.clone(),
                            status: Error::ActionFailed,
                            attempt,
                            at_ms: clock.now_ms(),
                            may_have_effects: permissions::sensitive(false, &spec),
                        });
                        if host.policy().revocation_revision() != revision {
                            return self.stop(Error::Cancelled, consent);
                        }
                        if error.retryable
                            && spec.action.idempotent
                            && attempt < self.options.max_retries
                        {
                            attempt += 1;
                            continue;
                        }
                        return self.stop(Error::ActionFailed, consent);
                    }
                    Err(error) => {
                        self.journal.push(Event {
                            node_id: node.id.clone(),
                            status: error.clone(),
                            attempt,
                            at_ms: clock.now_ms(),
                            may_have_effects: permissions::sensitive(false, &spec),
                        });
                        return self.stop(error, consent);
                    }
                }
            }
        }
        self.stop(Error::Completed, consent)
    }
}
async fn interruptible<T>(
    operation: impl Future<Output = T>,
    clock: &impl Clock,
    deadline: u64,
    cancel: &Cancellation,
) -> Result<T, Error> {
    let mut operation = std::pin::pin!(operation);
    let mut timer = std::pin::pin!(clock.wait_until(deadline));
    std::future::poll_fn(|cx| {
        if cancel.poll(cx).is_ready() {
            return Poll::Ready(Err(Error::Cancelled));
        }
        if clock.now_ms() >= deadline {
            return Poll::Ready(Err(Error::Timeout));
        }
        if let Poll::Ready(value) = operation.as_mut().poll(cx) {
            return Poll::Ready(if clock.now_ms() >= deadline {
                Err(Error::Timeout)
            } else {
                Ok(value)
            });
        }
        if timer.as_mut().poll(cx).is_ready() {
            return Poll::Ready(Err(Error::Timeout));
        }
        Poll::Pending
    })
    .await
}
