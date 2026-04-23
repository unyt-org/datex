use crate::{
    prelude::*, shared_values::errors::AccessError, types::error::TypeError,
};
use core::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum UpdateError {
    ImmutableValue,
    InvalidUpdate,
    AccessError(AccessError),
    TypeError(Box<TypeError>),
}

impl<T: Into<AccessError>> From<T> for UpdateError {
    fn from(err: T) -> Self {
        UpdateError::AccessError(err.into())
    }
}

impl From<TypeError> for UpdateError {
    fn from(err: TypeError) -> Self {
        UpdateError::TypeError(Box::new(err))
    }
}

impl Display for UpdateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            UpdateError::InvalidUpdate => {
                core::write!(f, "Invalid update operation")
            }
            UpdateError::AccessError(e) => {
                core::write!(f, "Access error: {}", e)
            }
            UpdateError::TypeError(e) => {
                core::write!(f, "Type error: {}", e)
            }
            UpdateError::ImmutableValue => {
                core::write!(f, "Cannot update an immutable value")
            }
        }
    }
}
