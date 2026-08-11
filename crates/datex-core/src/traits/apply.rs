use core::fmt::Display;

use crate::{
    runtime::Runtime,
    values::{
        core_values::callable::error::CallableError,
        value_container::ValueContainer,
    },
};
use alloc::boxed::Box;
use crate::prelude::*;

#[derive(Debug)]
pub enum ApplyError {
    UnsupportedApply,
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
    fn try_apply(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, ApplyError>;
    /// Applies a single ValueContainer argument to self
    fn try_apply_single(
        &self,
        runtime: &Runtime,
        arg: ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError>;
}
