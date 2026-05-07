use crate::{
    dif::cache::CacheValueRetrievalError,
    runtime::execution::ExecutionError,
    shared_values::errors::SharedValueCreationError,
    value_updates::{errors::UpdateError, update_data::UpdateReturn},
};
use core::{fmt::Display, result::Result};
use strum_macros::Display;
use crate::shared_values::base_shared_value_container::observers::ObserverError;

pub type DIFUpdateResult = Result<UpdateReturn, DIFUpdateError>;

/// Converts a Result with any types that can be converted into UpdateReturn and UpdateError into an UpdateResult.
pub fn into_update_result<T: Into<UpdateReturn>, E: Into<DIFUpdateError>>(
    result: Result<T, E>,
) -> DIFUpdateResult {
    match result {
        Ok(value) => Ok(value.into()),
        Err(err) => Err(err.into()),
    }
}

impl Display for DIFCreatePointerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DIFCreatePointerError::ReferenceNotFound => {
                core::write!(f, "Reference not found")
            }
            DIFCreatePointerError::ReferenceCreationError(e) => {
                core::write!(f, "Reference from value container error: {}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum DIFResolveReferenceError {
    ReferenceNotFound,
}
impl Display for DIFResolveReferenceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DIFResolveReferenceError::ReferenceNotFound => {
                core::write!(f, "Reference not found")
            }
        }
    }
}

impl From<SharedValueCreationError> for DIFCreatePointerError {
    fn from(err: SharedValueCreationError) -> Self {
        DIFCreatePointerError::ReferenceCreationError(err)
    }
}

#[derive(Debug)]
pub enum DIFObserveError {
    ReferenceNotFound,
    ObserveError(ObserverError),
}
impl From<ObserverError> for DIFObserveError {
    fn from(err: ObserverError) -> Self {
        DIFObserveError::ObserveError(err)
    }
}
impl Display for DIFObserveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DIFObserveError::ReferenceNotFound => {
                core::write!(f, "Reference not found")
            }
            DIFObserveError::ObserveError(e) => {
                core::write!(f, "Observe error: {}", e)
            }
        }
    }
}

#[derive(Debug, Display)]
pub enum DIFUpdateError {
    UpdateError(UpdateError),
    CacheValueRetrievalError(CacheValueRetrievalError),
}

impl From<UpdateError> for DIFUpdateError {
    fn from(err: UpdateError) -> Self {
        DIFUpdateError::UpdateError(err)
    }
}

impl From<CacheValueRetrievalError> for DIFUpdateError {
    fn from(err: CacheValueRetrievalError) -> Self {
        DIFUpdateError::CacheValueRetrievalError(err)
    }
}

#[derive(Debug)]
pub enum DIFApplyError {
    ExecutionError(ExecutionError),
    ReferenceNotFound,
}
impl Display for DIFApplyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DIFApplyError::ExecutionError(e) => {
                core::write!(f, "Execution error: {}", e)
            }
            DIFApplyError::ReferenceNotFound => {
                core::write!(f, "Reference not found")
            }
        }
    }
}

#[derive(Debug)]
pub enum DIFCreatePointerError {
    ReferenceNotFound,
    ReferenceCreationError(SharedValueCreationError),
}
