use core::marker::PhantomData;
use std::collections::HashMap;
use crate::shared_values::pointer_address::PointerAddress;
use crate::shared_values::shared_containers::{OwnedSharedContainer, ReferenceMutability, ReferencedSharedContainer, SharedContainer, SharedContainerOwnership};

/// Cache layer that stores references or owned and referenced shared containers used by the DIF client
/// during deserialization
#[derive(Debug, Default)]
pub struct DIFSharedContainerCache {
    values: HashMap<PointerAddress, SharedContainer>,
}

impl DIFSharedContainerCache {
    /// Stores a shared container in the cache, indexed by its pointer address. If a container already exists at the address,
    /// the container with the maximum ownership is kept.
    pub fn store_shared_container(&mut self, container: SharedContainer) {
        self.values.insert(container.pointer_address(), container);
    }

    /// Removes the shared container for the given pointer address from the cache, if it exists.
    pub fn remove_shared_container(&mut self, pointer_address: &PointerAddress) {
        self.values.remove(pointer_address);
    }

    /// Tries to take an owned shared container from the cache for the given pointer address.
    /// If the container for the address is not an owned shared container, or if there is no container for the address, None is returned and the cache is not modified.
    /// If an owned shared container is found for the address, it is replaced with a reference in the cache and the owned container is returned
    pub fn try_take_owned_shared_container(&mut self, pointer_address: &PointerAddress) -> Option<OwnedSharedContainer> {
        match self.values.get(pointer_address) {
            Some(container @ SharedContainer::Owned(..)) => {
                // replace owned with reference in cache and return owned
                match self.values.insert(pointer_address.clone(), SharedContainer::Referenced(container.derive_with_max_mutability())) {
                    Some(SharedContainer::Owned(owned_container)) => Some(owned_container),
                    _ => unreachable!(),
                }
            },
            _ => None,
        }
    }

    /// Tries to get a mutable reference to a shared container from the cache at the given pointer address.
    /// If the container for the address is not in the cache, or if the container cannot be accessed as a mutable reference, None is returned.
    pub fn try_get_shared_container_mutable_reference(&mut self, pointer_address: &PointerAddress) -> Option<ReferencedSharedContainer> {
        self.values.get(pointer_address)?.try_derive_mutable_reference().ok()
    }

    /// Tries to get an immutable reference to a shared container from the cache at the given pointer address.
    /// If the container for the address is not in the cache, None is returned.
    pub fn try_get_shared_container_immutable_reference(&mut self, pointer_address: &PointerAddress) -> Option<ReferencedSharedContainer> {
        Some(self.values.get(pointer_address)?.derive_immutable_reference())
    }

    /// Tries to get a shared container from the cache at the given pointer address with the specified ownership.
    /// If the container for the address is not in the cache, or if the container cannot be accessed with the specified ownership, None is returned.
    pub fn try_get_shared_container_with_ownership(&mut self, pointer_address: &PointerAddress, ownership: SharedContainerOwnership) -> Option<SharedContainer> {
        match ownership {
            SharedContainerOwnership::Owned => self.try_take_owned_shared_container(pointer_address).map(SharedContainer::Owned),
            SharedContainerOwnership::Referenced(ReferenceMutability::Mutable) => self.try_get_shared_container_mutable_reference(pointer_address).map(SharedContainer::Referenced),
            SharedContainerOwnership::Referenced(ReferenceMutability::Immutable) => self.try_get_shared_container_immutable_reference(pointer_address).map(SharedContainer::Referenced),
        }
    }
}


pub struct DeserializationContext<'ctx, T> {
    pub shared_container_cache: &'ctx mut DIFSharedContainerCache,
    _marker: PhantomData<T>,
}

impl<'ctx, T> DeserializationContext<'ctx, T> {
    pub fn new(shared_container_cache: &'ctx mut DIFSharedContainerCache) -> Self {
        Self { shared_container_cache, _marker: PhantomData }
    }

    // Converts this deserialization context to a deserialization context for another type U
    pub fn cast<U>(&mut self) -> DeserializationContext<'_, U> {
        DeserializationContext::new(self.shared_container_cache)
    }
}


