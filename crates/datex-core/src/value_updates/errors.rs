use core::fmt::Display;
use crate::shared_values::errors::{AccessError, AssignmentError};
use crate::type_inference::error::TypeError;

#[derive(Debug)]
pub enum UpdateError {
    ReferenceNotFound,
    InvalidUpdate,
    AccessError(AccessError),
    AssignmentError(AssignmentError),
    TypeError(Box<TypeError>),
}


impl From<AccessError> for UpdateError {
    fn from(err: AccessError) -> Self {
        UpdateError::AccessError(err)
    }
}
impl From<AssignmentError> for UpdateError {
    fn from(err: AssignmentError) -> Self {
        UpdateError::AssignmentError(err)
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
            UpdateError::ReferenceNotFound => {
                core::write!(f, "Reference not found")
            }
            UpdateError::InvalidUpdate => {
                core::write!(f, "Invalid update operation")
            }
            UpdateError::AccessError(e) => {
                core::write!(f, "Access error: {}", e)
            }
            UpdateError::AssignmentError(e) => {
                core::write!(f, "Assignment error: {}", e)
            }
            UpdateError::TypeError(e) => {
                core::write!(f, "Type error: {}", e)
            }
        }
    }
}