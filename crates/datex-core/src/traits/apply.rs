use core::fmt::Display;

use crate::{
    prelude::*,
    runtime::Runtime,
    values::{
        core_values::callable::error::CallableError,
        value_container::ValueContainer,
    },
};
use alloc::boxed::Box;

#[derive(Debug)]
pub enum ApplyError {
    UnsupportedApply,
    AsyncCallableRequiresAsyncExecution,
    CallableError(Box<CallableError>),
}
impl Display for ApplyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ApplyError::UnsupportedApply => {
                write!(f, "Value does not support apply operation")
            }
            ApplyError::CallableError(error) => {
                write!(f, "Error during callable application: {}", error)
            }
            ApplyError::AsyncCallableRequiresAsyncExecution => {
                write!(f, "Async callable requires async execution")
            }
        }
    }
}
impl From<CallableError> for ApplyError {
    fn from(error: CallableError) -> Self {
        ApplyError::CallableError(Box::new(error))
    }
}

// TODO #351: return ApplyErrors including call stack information (or store call stack directly in ExecutionError)
pub trait Apply {
    /// Applies multiple ValueContainer arguments to self
    /// Returns an Error if the value does not support sync apply
    fn try_apply_sync(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<(Vec<ValueContainer>, Option<ValueContainer>), ApplyError>;

    async fn try_apply_async(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<(Vec<ValueContainer>, Option<ValueContainer>), ApplyError>;
}
