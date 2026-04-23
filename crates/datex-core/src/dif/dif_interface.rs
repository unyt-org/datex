use serde::Deserialize;

use crate::{
    dif::pointer_address::PointerAddressWithOwnership,
    runtime::execution::ExecutionError,
    shared_values::{
        SharedContainerMutability,
        observers::{ObserveOptions, ObserverError, ObserverId, TransceiverId},
    },
};
use core::{fmt::Display, result::Result};

use crate::{
    dif::cache::DIFSharedContainerCache,
    shared_values::{
        PointerAddress, SelfOwnedPointerAddress,
        base_shared_value_container::BaseSharedValueContainer,
        errors::SharedValueCreationError,
    },
    types::r#type::Type,
    value_updates::update_data::{Update, UpdateData},
    values::value_container::ValueContainer,
};
use crate::dif::cache::CacheValueRetrievalError;
use crate::shared_values::SharedContainer;
use crate::value_updates::errors::UpdateError;
use crate::value_updates::update_data::{UpdateReturn};
use crate::value_updates::update_handler::UpdateHandler;

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

pub type DIFUpdateResult = Result<UpdateReturn, DIFUpdateError>;

/// Converts a Result with any types that can be converted into UpdateReturn and UpdateError into an UpdateResult.
pub fn into_update_result<T: Into<UpdateReturn>, E: Into<DIFUpdateError>>(result: Result<T, E>) -> DIFUpdateResult {
    match result {
        Ok(value) => Ok(value.into()),
        Err(err) => Err(err.into()),
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
    transceiver_id: TransceiverId,
}

impl DIFInterface {
    /// Applies a DIF update to the value at the given pointer address.
    fn update(
        &self,
        address: PointerAddress,
        update: Update,
    ) -> DIFUpdateResult {
        let container = self.cache.try_get_shared_container_mutable_reference(&address)?;
        let mut base_container = container.base_shared_container_mut();

        match update.data {
            UpdateData::AppendEntry(data) => into_update_result(base_container.try_append_entry(data, self.transceiver_id)),
            UpdateData::Clear => into_update_result( base_container.try_clear(self.transceiver_id)),
            UpdateData::Replace(data) => into_update_result(base_container.try_replace(data, self.transceiver_id)),
            UpdateData::SetEntry(data) => into_update_result(base_container.try_set_entry(data, self.transceiver_id)),
            UpdateData::DeleteEntry(data) => into_update_result(base_container.try_delete_entry(data, self.transceiver_id)),
            UpdateData::ListSplice(data) => into_update_result(base_container.try_list_splice(data, self.transceiver_id)),
        }
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
        _value: BaseSharedValueContainer,
    ) -> Result<SelfOwnedPointerAddress, DIFCreatePointerError> {
        todo!()
    }

    /// Resolves a pointer address of a pointer that is currently in memory.
    /// Returns an error if the pointer is not found in memory.
    fn resolve_pointer_address(
        &self,
        _address: PointerAddressWithOwnership,
    ) -> Result<BaseSharedValueContainer, DIFResolveReferenceError> {
        todo!()
    }

    /// Starts observing changes to the pointer at the given address.
    /// As long as the pointer is observed, it will not be garbage collected.
    fn observe_pointer(
        &self,
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
        _observer_id: ObserverId,
        _options: ObserveOptions,
    ) -> Result<(), DIFObserveError> {
        todo!()
    }

    /// Stops observing changes to the pointer at the given address.
    /// If no other references to the pointer exist, it may be garbage collected after this call.
    fn unobserve_pointer(
        &self,
        _address: PointerAddress,
        _observer_id: ObserverId,
    ) -> Result<(), DIFObserveError> {
        todo!()
    }

    // TODO: lock/unlock pointers
}
