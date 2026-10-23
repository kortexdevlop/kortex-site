use crate::errors::RuntimeError;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Created,
    Loaded,
    Initialized,
    Running,
    Stopped,
    Error,
}
impl State {
    pub fn transition(&mut self, next: Self) -> Result<(), RuntimeError> {
        let valid = matches!(
            (*self, next),
            (Self::Created, Self::Loaded)
                | (Self::Loaded, Self::Initialized)
                | (Self::Initialized, Self::Running)
        ) || (!matches!(*self, Self::Stopped | Self::Error)
            && matches!(next, Self::Stopped | Self::Error));
        if !valid {
            return Err(RuntimeError::InvalidState);
        }
        *self = next;
        Ok(())
    }
}
