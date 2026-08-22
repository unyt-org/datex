use crate::{
    runtime::{
        cache::shared_values_cache::CacheValueRetrievalError,
        execution::ExecutionError,
    },
    shared_values::base_shared_value_container::observers::ObserverError,
    value_updates::errors::UpdateError,
};
use core::fmt::Display;

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

#[derive(Debug)]
pub enum DIFUpdateError {
    UpdateError(UpdateError),
    CacheValueRetrievalError(CacheValueRetrievalError),
}

impl Display for DIFUpdateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DIFUpdateError::UpdateError(e) => {
                core::write!(f, "Update error: {}", e)
            }
            DIFUpdateError::CacheValueRetrievalError(e) => {
                core::write!(f, "Cache value retrieval error: {}", e)
            }
        }
    }
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
