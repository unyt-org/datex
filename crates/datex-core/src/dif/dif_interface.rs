use crate::{
    dif::pointer_address::{self, PointerAddressWithOwnership},
    runtime::{
        execution::ExecutionError,
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::{
        OwnedSharedContainer, SelfOwnedSharedContainer, SharedContainer,
        observers::{
            ObserveOptions, Observer, ObserverError, ObserverId, TransceiverId,
        },
    },
    traits::apply::Apply,
    values::core_values::endpoint::Endpoint,
};
use alloc::rc::Rc;
use core::{fmt::Display, result::Result};
use core::cell::RefCell;
use crate::{
    dif::cache::{CacheValueRetrievalError, DIFSharedContainerCache},
    shared_values::{
        PointerAddress, SelfOwnedPointerAddress,
        base_shared_value_container::BaseSharedValueContainer,
        errors::SharedValueCreationError,
    },
    value_updates::{
        errors::UpdateError,
        update_data::{Update, UpdateData, UpdateReturn},
        update_handler::UpdateHandler,
    },
    values::value_container::ValueContainer,
};
use crate::runtime::Runtime;

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
pub fn into_update_result<T: Into<UpdateReturn>, E: Into<DIFUpdateError>>(
    result: Result<T, E>,
) -> DIFUpdateResult {
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
    address_provider: Rc<RefCell<SelfOwnedPointerAddressProvider>>,
    transceiver_id: TransceiverId,
}

impl DIFInterface {
    pub fn new(transceiver_id: TransceiverId, address_provider: Rc<RefCell<SelfOwnedPointerAddressProvider>>) -> Self {
        DIFInterface {
            cache: DIFSharedContainerCache::default(),
            address_provider,
            transceiver_id,
        }
    }
}
impl DIFInterface {
    /// Applies a DIF update to the value at the given pointer address.
    pub fn update(
        &self,
        address: PointerAddress,
        update: Update,
    ) -> DIFUpdateResult {
        let container = self
            .cache
            .try_get_shared_container_mutable_reference(&address)?;
        let mut base_container = container.base_shared_container_mut();

        match update.data {
            UpdateData::AppendEntry(data) => into_update_result(
                base_container.try_append_entry(data, self.transceiver_id),
            ),
            UpdateData::Clear => into_update_result(
                base_container.try_clear(self.transceiver_id),
            ),
            UpdateData::Replace(data) => into_update_result(
                base_container.try_replace(data, self.transceiver_id),
            ),
            UpdateData::SetEntry(data) => into_update_result(
                base_container.try_set_entry(data, self.transceiver_id),
            ),
            UpdateData::DeleteEntry(data) => into_update_result(
                base_container.try_delete_entry(data, self.transceiver_id),
            ),
            UpdateData::ListSplice(data) => into_update_result(
                base_container.try_list_splice(data, self.transceiver_id),
            ),
        }
    }

    /// Executes an apply operation, applying the `value` to the `callee`.
    pub fn apply(
        &self,
        callee: ValueContainer,
        value: ValueContainer,
    ) -> Result<Option<ValueContainer>, DIFApplyError> {
        todo!()
    }

    /// Creates a new owned local pointer and stores it in memory.
    /// Returns the address of the newly created pointer.
    pub fn create_pointer(
        &mut self,
        value: BaseSharedValueContainer,
    ) -> Result<SelfOwnedPointerAddress, DIFCreatePointerError> {
        let pointer_address =
            self.address_provider.borrow_mut().get_new_self_owned_address();
        self.cache.store_shared_container(SharedContainer::Owned(
            OwnedSharedContainer::new_from_self_owned_container(
                SelfOwnedSharedContainer::new(value, pointer_address.clone()),
            ),
        ));
        Ok(pointer_address)
    }

    /// Resolves a pointer address of a pointer that is currently in memory.
    /// Returns an error if the pointer is not found in memory.
    pub fn resolve_pointer_address(
        &mut self,
        address_with_ownership: PointerAddressWithOwnership,
    ) -> Result<BaseSharedValueContainer, DIFResolveReferenceError> {
        let container = self
            .cache
            .try_get_shared_container_with_ownership(
                &address_with_ownership.address,
                address_with_ownership.ownership,
            )
            .map_err(|_| DIFResolveReferenceError::ReferenceNotFound)?;
        todo!()
    }

    /// Starts observing changes to the pointer at the given address.
    /// As long as the pointer is observed, it will not be garbage collected.
    pub fn observe_pointer(
        &self,
        address: PointerAddress,
        options: ObserveOptions,
        callback: impl Fn(&Update) + 'static,
    ) -> Result<ObserverId, DIFObserveError> {
        let shared_container_ref = self
            .cache
            .try_get_shared_container(&address)
            .map_err(|_| DIFObserveError::ReferenceNotFound)?;
        Ok(shared_container_ref.base_shared_container_mut().observe(
            Observer {
                transceiver_id: self.transceiver_id,
                options,
                callback: Rc::new(callback),
            },
        )?)
    }

    /// Updates the options for an existing observer on the pointer at the given address.
    /// If the observer does not exist, an error is returned.
    pub fn update_observer_options(
        &self,
        address: PointerAddress,
        observer_id: ObserverId,
        options: ObserveOptions,
    ) -> Result<(), DIFObserveError> {
        let shared_container_ref = self
            .cache
            .try_get_shared_container(&address)
            .map_err(|_| DIFObserveError::ReferenceNotFound)?;
        shared_container_ref
            .base_shared_container_mut()
            .update_observer_options(observer_id, options)?;
        Ok(())
    }

    /// Stops observing changes to the pointer at the given address.
    /// If no other references to the pointer exist, it may be garbage collected after this call.
    pub fn unobserve_pointer(
        &self,
        address: PointerAddress,
        observer_id: ObserverId,
    ) -> Result<(), DIFObserveError> {
        let shared_container_ref = self
            .cache
            .try_get_shared_container(&address)
            .map_err(|_| DIFObserveError::ReferenceNotFound)?;
        shared_container_ref
            .base_shared_container_mut()
            .unobserve(observer_id)?;
        Ok(())
    }

    // TODO: lock/unlock pointers
}
