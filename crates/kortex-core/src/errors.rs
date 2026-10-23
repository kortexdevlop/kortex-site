use kortex_abi::HostError;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    UnknownInstance,
    InvalidState,
    QueueFull,
    QueueBusy,
    InvalidInput,
    InvalidOutput,
    UnsupportedAbi,
    IntegrityError,
    UntrustedKey,
    SignatureInvalid,
    Timeout,
    GuestCrash,
    GuestError(HostError),
    CorrelationError,
    InvalidReply,
    Engine(String),
    CounterExhausted,
    CleanupFailure {
        cause: Box<RuntimeError>,
        cleanup: Box<RuntimeError>,
    },
}
impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for RuntimeError {}
