use crate::{
    collections::HashMap,
    shared_values::{
        pointer_address::PointerAddress,
        shared_containers::{
            OwnedSharedContainer, ReferenceMutability,
            ReferencedSharedContainer, SharedContainer,
            SharedContainerOwnership,
            errors::{
                UnexpectedImmutableReferenceError,
                UnexpectedSharedContainerOwnershipError,
            },
        },
    },
};
use strum_macros::Display;

/// Cache layer that stores references or owned and referenced shared containers used by the DIF client
/// during deserialization
#[derive(Debug, Default)]
pub struct DIFSharedContainerCache {
    values: HashMap<PointerAddress, SharedContainer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueNotFoundInCacheError;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum CacheValueRetrievalError {
    UnexpectedImmutableReference(UnexpectedImmutableReferenceError),
    UnexpectedSharedContainerOwnership(UnexpectedSharedContainerOwnershipError),
    ValueNotFoundInCache(ValueNotFoundInCacheError),
}

impl From<UnexpectedImmutableReferenceError> for CacheValueRetrievalError {
    fn from(error: UnexpectedImmutableReferenceError) -> Self {
        CacheValueRetrievalError::UnexpectedImmutableReference(error)
    }
}

impl From<ValueNotFoundInCacheError> for CacheValueRetrievalError {
    fn from(error: ValueNotFoundInCacheError) -> Self {
        CacheValueRetrievalError::ValueNotFoundInCache(error)
    }
}

impl From<UnexpectedSharedContainerOwnershipError>
    for CacheValueRetrievalError
{
    fn from(error: UnexpectedSharedContainerOwnershipError) -> Self {
        CacheValueRetrievalError::UnexpectedSharedContainerOwnership(error)
    }
}

impl DIFSharedContainerCache {
    /// Stores a shared container in the cache, indexed by its pointer address. If a container already exists at the address,
    /// the container with the maximum ownership is kept.
    pub fn store_shared_container(&mut self, container: SharedContainer) {
        self.values.insert(container.pointer_address(), container);
    }

    /// Removes the shared container for the given pointer address from the cache, if it exists.
    pub fn remove_shared_container(
        &mut self,
        pointer_address: &PointerAddress,
    ) {
        self.values.remove(pointer_address);
    }

    /// Tries to take an owned shared container from the cache for the given pointer address.
    /// If the container for the address is not an owned shared container, or if there is no container for the address, None is returned and the cache is not modified.
    /// If an owned shared container is found for the address, it is replaced with a reference in the cache and the owned container is returned
    pub fn try_take_owned_shared_container(
        &mut self,
        pointer_address: &PointerAddress,
    ) -> Result<OwnedSharedContainer, CacheValueRetrievalError> {
        match self.values.get(pointer_address) {
            Some(container) => {
                match container {
                    SharedContainer::Owned(container) => {
                        // replace owned with reference in cache and return owned
                        match self.values.insert(
                            pointer_address.clone(),
                            SharedContainer::Referenced(
                                container.derive_with_max_mutability(),
                            ),
                        ) {
                            Some(SharedContainer::Owned(owned_container)) => {
                                Ok(owned_container)
                            }
                            _ => unreachable!(),
                        }
                    }
                    SharedContainer::Referenced(reference) => {
                        Err(UnexpectedSharedContainerOwnershipError {
                            actual: SharedContainerOwnership::Referenced(
                                reference.reference_mutability(),
                            ),
                            expected: SharedContainerOwnership::Owned,
                        }
                        .into())
                    }
                }
            }
            _ => Err(ValueNotFoundInCacheError.into()),
        }
    }

    pub fn try_get_shared_container(
        &self,
        pointer_address: &PointerAddress,
    ) -> Result<&SharedContainer, ValueNotFoundInCacheError> {
        self.values
            .get(pointer_address)
            .ok_or(ValueNotFoundInCacheError)
    }

    /// Tries to get a mutable reference to a shared container from the cache at the given pointer address.
    /// If the container for the address is not in the cache, a [ValueNotFoundInCacheError] is returned.
    /// If the container cannot be accessed as a mutable reference, an [UnexpectedImmutableReferenceError] error is returned.
    pub fn try_get_shared_container_mutable_reference(
        &mut self,
        pointer_address: &PointerAddress,
    ) -> Result<ReferencedSharedContainer, CacheValueRetrievalError> {
        Ok(self
            .values
            .get(pointer_address)
            .ok_or(ValueNotFoundInCacheError)?
            .try_derive_mutable_reference()?)
    }

    /// Tries to get an immutable reference to a shared container from the cache at the given pointer address.
    /// If the container for the address is not in the cache, a [ValueNotFoundInCacheError] is returned.
    pub fn try_get_shared_container_immutable_reference(
        &mut self,
        pointer_address: &PointerAddress,
    ) -> Result<ReferencedSharedContainer, ValueNotFoundInCacheError> {
        Ok(self
            .values
            .get(pointer_address)
            .ok_or(ValueNotFoundInCacheError)?
            .derive_immutable_reference())
    }

    /// Tries to get a shared container from the cache at the given pointer address with the specified ownership.
    /// If the container for the address is not in the cache, or if the container cannot be accessed with the specified ownership, a [CacheValueRetrievalError] is returned.
    pub fn try_get_shared_container_with_ownership(
        &mut self,
        pointer_address: &PointerAddress,
        ownership: SharedContainerOwnership,
    ) -> Result<SharedContainer, CacheValueRetrievalError> {
        match ownership {
            SharedContainerOwnership::Owned => self
                .try_take_owned_shared_container(pointer_address)
                .map(SharedContainer::Owned),
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Mutable,
            ) => self
                .try_get_shared_container_mutable_reference(pointer_address)
                .map(SharedContainer::Referenced),
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Immutable,
            ) => Ok(self
                .try_get_shared_container_immutable_reference(pointer_address)
                .map(SharedContainer::Referenced)?),
        }
    }
}
