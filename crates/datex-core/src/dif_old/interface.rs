use crate::{
    dif::{
        update::DIFUpdateData,
        value::{DIFReferenceNotFoundError},
    },
    runtime::execution::ExecutionError,
    shared_values::{
        shared_containers::observers::{ObserveOptions, ObserverError, TransceiverId},
        shared_containers::{
            SharedContainerMutability,
        },
    },
};
use core::{fmt::Display, result::Result};

use crate::{prelude::*, shared_values::pointer_address::PointerAddress};
use crate::shared_values::errors::{AccessError, AssignmentError, SharedValueCreationError, TypeError};
use crate::shared_values::pointer_address::SelfOwnedPointerAddress;
use crate::shared_values::SharedContainer;
use crate::types::r#type::Type;
use crate::values::value_container::ValueContainer;

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
    ReferenceNotFound,
    InvalidUpdate,
    AccessError(AccessError),
    AssignmentError(AssignmentError),
    TypeError(Box<TypeError>),
}

impl From<DIFReferenceNotFoundError> for DIFUpdateError {
    fn from(_: DIFReferenceNotFoundError) -> Self {
        DIFUpdateError::ReferenceNotFound
    }
}
impl From<AccessError> for DIFUpdateError {
    fn from(err: AccessError) -> Self {
        DIFUpdateError::AccessError(err)
    }
}
impl From<AssignmentError> for DIFUpdateError {
    fn from(err: AssignmentError) -> Self {
        DIFUpdateError::AssignmentError(err)
    }
}
impl From<TypeError> for DIFUpdateError {
    fn from(err: TypeError) -> Self {
        DIFUpdateError::TypeError(Box::new(err))
    }
}

impl Display for DIFUpdateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DIFUpdateError::ReferenceNotFound => {
                core::write!(f, "Reference not found")
            }
            DIFUpdateError::InvalidUpdate => {
                core::write!(f, "Invalid update operation")
            }
            DIFUpdateError::AccessError(e) => {
                core::write!(f, "Access error: {}", e)
            }
            DIFUpdateError::AssignmentError(e) => {
                core::write!(f, "Assignment error: {}", e)
            }
            DIFUpdateError::TypeError(e) => {
                core::write!(f, "Type error: {}", e)
            }
        }
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

impl From<DIFReferenceNotFoundError> for DIFCreatePointerError {
    fn from(_: DIFReferenceNotFoundError) -> Self {
        DIFCreatePointerError::ReferenceNotFound
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

pub trait DIFInterface {
    /// Applies a DIF update to the value at the given pointer address.
    fn update(
        &self,
        source_id: TransceiverId,
        address: PointerAddress,
        update: &DIFUpdateData,
    ) -> Result<(), DIFUpdateError>;

    /// Executes an apply operation, applying the `value` to the `callee`.
    fn apply(
        &self,
        callee: ValueContainer,
        value: ValueContainer,
    ) -> Result<ValueContainer, DIFApplyError>;

    /// Creates a new owned local pointer and stores it in memory.
    /// Returns the address of the newly created pointer.
    fn create_pointer(
        &self,
        value: ValueContainer,
        allowed_type: Option<Type>,
        mutability: SharedContainerMutability,
    ) -> Result<SelfOwnedPointerAddress, DIFCreatePointerError>;

    /// Resolves a pointer address of a pointer that is currently in memory.
    /// Returns an error if the pointer is not found in memory.
    fn resolve_pointer_address(
        &self,
        address: PointerAddress,
    ) -> Result<SharedContainer, DIFResolveReferenceError>;

    /// Starts observing changes to the pointer at the given address.
    /// As long as the pointer is observed, it will not be garbage collected.
    fn observe_pointer(
        &self,
        transceiver_id: TransceiverId,
        address: PointerAddress,
        options: ObserveOptions,
        observer: impl Fn(&DIFUpdateData, TransceiverId) + 'static,
    ) -> Result<u32, DIFObserveError>;

    /// Updates the options for an existing observer on the pointer at the given address.
    /// If the observer does not exist, an error is returned.
    fn update_observer_options(
        &self,
        address: PointerAddress,
        observer_id: u32,
        options: ObserveOptions,
    ) -> Result<(), DIFObserveError>;

    /// Stops observing changes to the pointer at the given address.
    /// If no other references to the pointer exist, it may be garbage collected after this call.
    fn unobserve_pointer(
        &self,
        address: PointerAddress,
        observer_id: u32,
    ) -> Result<(), DIFObserveError>;
}
