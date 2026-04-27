use crate::{
    dif::{
        cache::DIFSharedContainerCache,
        error::{
            DIFApplyError, DIFCreatePointerError, DIFObserveError,
            DIFResolveReferenceError, DIFUpdateResult, into_update_result,
        },
        pointer_address::PointerAddressWithOwnership,
    },
    runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
    shared_values::{
        OwnedSharedContainer, PointerAddress, SelfOwnedPointerAddress,
        SelfOwnedSharedContainer, SharedContainer,
        base_shared_value_container::BaseSharedValueContainer,
        observers::{ObserveOptions, Observer, ObserverId, TransceiverId},
    },
    value_updates::{
        update_data::{Update, UpdateData},
        update_handler::UpdateHandler,
    },
    values::value_container::ValueContainer,
};
use alloc::rc::Rc;
use core::{cell::RefCell, result::Result};

pub struct DIFInterface {
    pub cache: DIFSharedContainerCache,
    address_provider: Rc<RefCell<SelfOwnedPointerAddressProvider>>,
    transceiver_id: TransceiverId,
}

impl DIFInterface {
    pub fn new(
        transceiver_id: TransceiverId,
        address_provider: Rc<RefCell<SelfOwnedPointerAddressProvider>>,
    ) -> Self {
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
        _callee: ValueContainer,
        _value: ValueContainer,
    ) -> Result<Option<ValueContainer>, DIFApplyError> {
        todo!()
    }

    /// Creates a new owned local pointer and stores it in memory.
    /// Returns the [SelfOwnedPointerAddress] of the newly created pointer.
    pub fn create_pointer(
        &mut self,
        value: BaseSharedValueContainer,
    ) -> Result<SelfOwnedPointerAddress, DIFCreatePointerError> {
        let pointer_address = self
            .address_provider
            .borrow_mut()
            .get_new_self_owned_address();
        self.cache.store_shared_container(SharedContainer::Owned(
            OwnedSharedContainer::new_from_self_owned_container(
                SelfOwnedSharedContainer::new(value, pointer_address.clone()),
            ),
        ));
        Ok(pointer_address)
    }

    /// Resolves a pointer address of a pointer that is currently in memory to its [SharedContainer].
    /// Returns an error if the pointer is not found in memory.
    pub fn resolve_pointer_address(
        &mut self,
        address_with_ownership: PointerAddressWithOwnership,
    ) -> Result<SharedContainer, DIFResolveReferenceError> {
        self.cache
            .try_get_shared_container_with_ownership(
                &address_with_ownership.address,
                address_with_ownership.ownership,
            )
            .map_err(|_| DIFResolveReferenceError::ReferenceNotFound)
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
