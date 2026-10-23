use crate::errors::RuntimeError;
use kortex_abi::{Envelope, MAX_ACTION_MS, MAX_QUEUED_EVENTS};
use std::{collections::VecDeque, future::Future, task::Poll};
pub struct Queued {
    pub request: Envelope,
    pub timeout_ms: u32,
    pub action: Option<String>,
}
#[derive(Default)]
pub struct Dispatcher {
    queue: VecDeque<Queued>,
}
impl Dispatcher {
    pub fn push(&mut self, value: Queued) -> Result<(), RuntimeError> {
        if self.queue.len() >= MAX_QUEUED_EVENTS {
            return Err(RuntimeError::QueueFull);
        }
        if value.timeout_ms == 0
            || value.timeout_ms > MAX_ACTION_MS
            || value.request.encode().is_err()
        {
            return Err(RuntimeError::InvalidInput);
        }
        self.queue.push_back(value);
        Ok(())
    }
    pub fn pop(&mut self) -> Option<Queued> {
        self.queue.pop_front()
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
/// Host adapters supply real monotonic time and wake the timeout future.
/// Dropping a pending engine future must cancel its work before unloading its handle.
pub trait Clock {
    fn now_ms(&self) -> u64;
    fn wait_until(&self, deadline_ms: u64) -> impl Future<Output = ()>;
}
pub async fn bounded<T>(
    operation: impl Future<Output = Result<T, RuntimeError>>,
    clock: &impl Clock,
    deadline: u64,
) -> Result<T, RuntimeError> {
    let mut operation = std::pin::pin!(operation);
    let mut timer = std::pin::pin!(clock.wait_until(deadline));
    std::future::poll_fn(|cx| {
        if clock.now_ms() >= deadline {
            return Poll::Ready(Err(RuntimeError::Timeout));
        }
        if let Poll::Ready(value) = operation.as_mut().poll(cx) {
            return Poll::Ready(if clock.now_ms() >= deadline {
                Err(RuntimeError::Timeout)
            } else {
                value
            });
        }
        if timer.as_mut().poll(cx).is_ready() {
            return Poll::Ready(Err(RuntimeError::Timeout));
        }
        Poll::Pending
    })
    .await
}

/// Every sensitive request from an engine adapter goes through this policy gateway.
/// Identity and declarations must be supplied from the loaded instance, never guest JSON.
pub async fn execute_sensitive<H: kortex_capabilities::CapabilityHost>(
    gateway: &mut kortex_capabilities::Gateway<H>,
    context: &kortex_capabilities::policy::Context,
    declared: &[String],
    request: kortex_capabilities::CapabilityCall,
    gesture: Option<&kortex_capabilities::policy::GestureToken>,
    time: &impl Fn() -> u64,
) -> Result<kortex_capabilities::CapabilityResponse, kortex_capabilities::CapabilityError> {
    let permissions = declared
        .iter()
        .map(|name| kortex_capabilities::Capability::try_from(name.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    gateway.policy.declare(context.clone(), permissions);
    gateway.execute(context, request, gesture, time).await
}
