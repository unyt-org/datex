use crate::{
    runtime::execution::ExecutionError,
    shared_values::shared_containers::{
        SharedContainerMutability,
        observers::{ObserveOptions, ObserverError, TransceiverId},
    },
};
use core::{fmt::Display, result::Result};

use crate::{
    dif::cache::DIFSharedContainerCache,
    shared_values::{
        errors::SharedValueCreationError,
        pointer_address::{PointerAddress, SelfOwnedPointerAddress},
        shared_containers::base_shared_value_container::BaseSharedValueContainer,
    },
    types::r#type::Type,
    value_updates::update_data::{Update, UpdateData, UpdateResult},
    values::value_container::ValueContainer,
};

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

pub struct DIFInterface {
    cache: DIFSharedContainerCache,
}

impl DIFInterface {
    /// Applies a DIF update to the value at the given pointer address.
    fn update(
        &self,
        _address: PointerAddress,
        _update: &Update,
    ) -> UpdateResult {
        todo!()
        //self.cache.try_get_shared_container
    }

    /// Executes an apply operation, applying the `value` to the `callee`.
    fn apply(
        &self,
        _callee: ValueContainer,
        _value: ValueContainer,
    ) -> Result<ValueContainer, DIFApplyError> {
        todo!()
    }

    /// Creates a new owned local pointer and stores it in memory.
    /// Returns the address of the newly created pointer.
    fn create_pointer(
        &self,
        _value: ValueContainer,
        _allowed_type: Option<Type>,
        _mutability: SharedContainerMutability,
    ) -> Result<SelfOwnedPointerAddress, DIFCreatePointerError> {
        todo!()
    }

    /// Resolves a pointer address of a pointer that is currently in memory.
    /// Returns an error if the pointer is not found in memory.
    fn resolve_pointer_address(
        &self,
        _address: PointerAddress,
    ) -> Result<BaseSharedValueContainer, DIFResolveReferenceError> {
        todo!()
    }

    /// Starts observing changes to the pointer at the given address.
    /// As long as the pointer is observed, it will not be garbage collected.
    fn observe_pointer(
        &self,
        _transceiver_id: TransceiverId,
        _address: PointerAddress,
        _options: ObserveOptions,
        _observer: impl Fn(&UpdateData) + 'static,
    ) -> Result<u32, DIFObserveError> {
        todo!()
    }

    /// Updates the options for an existing observer on the pointer at the given address.
    /// If the observer does not exist, an error is returned.
    fn update_observer_options(
        &self,
        _address: PointerAddress,
        _observer_id: u32,
        _options: ObserveOptions,
    ) -> Result<(), DIFObserveError> {
        todo!()
    }

    /// Stops observing changes to the pointer at the given address.
    /// If no other references to the pointer exist, it may be garbage collected after this call.
    fn unobserve_pointer(
        &self,
        _address: PointerAddress,
        _observer_id: u32,
    ) -> Result<(), DIFObserveError> {
        todo!()
    }

    // TODO: lock/unlock pointers
}
