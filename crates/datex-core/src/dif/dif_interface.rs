use crate::{
    dif::{
        cache::{DIFSharedContainerCache, ValueNotFoundInCacheError},
        error::{DIFObserveError, DIFUpdateError},
    },
    prelude::*,
    runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
    shared_values::{
        OwnedSharedContainer, PointerAddress, SelfOwnedPointerAddress,
        SelfOwnedSharedContainer, SharedContainer, SharedContainerOwnership,
        base_shared_value_container::{
            BaseSharedValueContainer,
            observers::{
                ObserveOptions, Observer, ObserverCallback, ObserverId,
                TransceiverId,
            },
        },
    },
    traits::apply::{Apply, ApplyError},
    value_updates::{
        UpdateReturn, update_data::Update, update_handler::UpdateHandler,
    },
    values::value_container::ValueContainer,
};
use alloc::rc::Rc;
use core::{cell::RefCell, result::Result};

pub type DIFUpdateResult = Result<UpdateReturn, DIFUpdateError>;

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
    /// Updates the shared container for the given address and returns an update result
    pub fn update(
        &self,
        address: &PointerAddress,
        update: Update,
    ) -> Result<UpdateReturn, DIFUpdateError> {
        let shared_container = self
            .cache
            .try_get_shared_container_mutable_reference(address)?;
        let mut base_container = shared_container.base_shared_container_mut();

        base_container
            .update(update)
            .map_err(DIFUpdateError::UpdateError)
    }

    /// Returns a list of all [ObserverCallback]s that are currently active for the pointer
    pub fn get_current_observers(
        &self,
        address: &PointerAddress,
        source_id: TransceiverId,
    ) -> Result<Vec<ObserverCallback>, ValueNotFoundInCacheError> {
        let shared_container = self
            .cache
            .try_get_shared_container_immutable_reference(address)?;
        let base_container = shared_container.base_shared_container();
        Ok(base_container.get_current_observers(source_id))
    }

    /// Executes an apply operation, applying the `value` to the `callee`.
    pub fn apply(
        &self,
        callee: ValueContainer,
        value: ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        callee.try_apply_single(&value)
    }

    /// Creates a new owned local pointer and stores it in memory.
    /// Returns the [SelfOwnedPointerAddress] of the newly created pointer.
    pub fn create_pointer(
        &mut self,
        value: BaseSharedValueContainer,
    ) -> SelfOwnedPointerAddress {
        let pointer_address = self
            .address_provider
            .borrow_mut()
            .get_new_self_owned_address();
        self.cache.store_shared_container(SharedContainer::Owned(
            OwnedSharedContainer::new_from_self_owned_container(
                SelfOwnedSharedContainer::new(value, pointer_address.clone()),
            ),
        ));
        pointer_address
    }

    /// Resolves a pointer address of a pointer that is currently in memory to its [SharedContainer].
    /// Returns an error if the pointer is not found in memory.
    pub fn resolve_pointer_address(
        &mut self,
        address: PointerAddress,
    ) -> Result<SharedContainer, ValueNotFoundInCacheError> {
        self.cache.try_get_shared_container(&address).cloned()
    }

    pub fn has_address_with_ownership(
        &self,
        pointer_address: &PointerAddress,
        ownership: SharedContainerOwnership,
    ) -> bool {
        self.cache
            .has_address_with_ownership(pointer_address, ownership)
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
