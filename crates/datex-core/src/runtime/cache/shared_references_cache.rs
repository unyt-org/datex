use crate::{
    collections::HashMap,
    shared_values::{
        PointerAddress, ReferencedSharedContainer, SharedContainer,
        weak_shared_container::WeakSharedContainer,
    },
    values::value_container::ValueContainer,
};
use core::fmt::Display;

#[derive(Debug, Default)]
pub struct SharedReferencesCache {
    /// References to owned values that are currently referenced and required by remote endpoints
    owned_values: HashMap<PointerAddress, ReferencedSharedContainer>,
    /// Weak references to remote values that this endpoint is currently subscribed to and receives updates for
    remote_values: HashMap<PointerAddress, WeakSharedContainer>,
}

impl SharedReferencesCache {
    /// Registers a remote shared container in the cache.
    /// This method should be called for shared containers that this endpoint is currently subscribed to and receives updates for.
    /// If the reference is already registered (has a PointerAddress), the existing address is returned and no new registration is done.
    /// The reference is stored as a weak reference, so it will be dropped when there are no more strong references to it.
    /// TODO: call remove on Rc drop!
    pub fn register_remote_shared_container(
        &mut self,
        container: &ReferencedSharedContainer,
    ) {
        let pointer_address = container.pointer_address();
        // check if reference is already registered (if it has an address, we assume it is registered)
        self.remote_values
            .entry(pointer_address)
            .or_insert_with(|| container.downgrade());
    }

    /// Registers an owned shared container in the cache.
    /// This method should be called for shared containers that this endpoint owns and is responsible for.
    /// If the reference is already registered (has a PointerAddress), the existing address is returned and no new registration is done.
    /// The reference is stored as a strong reference, so it will not be dropped until it is explicitly removed from the cache.
    pub fn register_owned_shared_container(
        &mut self,
        container: &ReferencedSharedContainer,
    ) {
        let pointer_address = container.pointer_address();
        // check if reference is already registered (if it has an address, we assume it is registered)
        self.owned_values
            .entry(pointer_address)
            .or_insert_with(|| container.clone());
    }

    /// Removes a remote shared container from the cache.
    pub fn remove_remote_shared_container(
        &mut self,
        pointer_address: &PointerAddress,
    ) {
        self.remote_values.remove(pointer_address);
    }

    /// Removes an owned shared container from the cache.
    pub fn remove_owned_shared_container(
        &mut self,
        pointer_address: &PointerAddress,
    ) {
        self.owned_values.remove(pointer_address);
    }

    /// Returns a reference stored at the given PointerAddress, if it exists.
    pub fn get_reference(
        &self,
        pointer_address: &PointerAddress,
    ) -> Option<ReferencedSharedContainer> {
        self.owned_values.get(pointer_address).cloned().or_else(|| {
            self.remote_values
                .get(pointer_address)
                .and_then(|weak_ref| weak_ref.upgrade())
        })
    }

    /// Checks if a reference with the given PointerAddress exists in memory.
    pub fn has_reference(&self, pointer_address: &PointerAddress) -> bool {
        self.owned_values.contains_key(pointer_address)
            || self
                .remote_values
                .get(pointer_address)
                .map_or(false, |weak_ref| weak_ref.upgrade().is_some())
    }

    /// Returns an iterator over all currently stored references in the cache, both owned and remote.
    pub fn values(&self) -> impl Iterator<Item = ReferencedSharedContainer> {
        gen move {
            for value in self.owned_values() {
                yield value.clone();
            }
            for value in self.remote_values() {
                yield value;
            }
        }
    }

    /// Returns an iterator over all currently stored owned references in the cache.
    pub fn owned_values(
        &self,
    ) -> impl Iterator<Item = &ReferencedSharedContainer> {
        self.owned_values.values()
    }

    /// Returns an iterator over all currently stored remote references in the cache.
    pub fn remote_values(
        &self,
    ) -> impl Iterator<Item = ReferencedSharedContainer> {
        self.remote_values
            .values()
            .filter_map(|weak_ref| weak_ref.upgrade())
    }

    /// Returns an iterator over all currently stored remote weak references in the cache.
    pub fn remote_values_weak(
        &self,
    ) -> impl Iterator<Item = &WeakSharedContainer> {
        self.remote_values.values()
    }
}

#[cfg(feature = "decompiler")]
impl Display for SharedReferencesCache {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use crate::decompiler::*;

        // print owned values
        writeln!(f, "Owned Values:")?;
        for (address, value) in &self.owned_values {
            writeln!(
                f,
                "  {}: {}",
                address,
                decompile_value(
                    &ValueContainer::Shared(SharedContainer::Referenced(
                        value.clone()
                    )),
                    DecompileOptions::default()
                )
            )?;
        }
        // print remote values
        writeln!(f, "Remote Values:")?;
        for (address, weak_value) in &self.remote_values {
            let value = weak_value.upgrade();
            if let Some(value) = value {
                writeln!(
                    f,
                    "  {}: {}",
                    address,
                    decompile_value(
                        &ValueContainer::Shared(SharedContainer::Referenced(
                            value
                        )),
                        DecompileOptions::default()
                    )
                )?;
            } else {
                writeln!(f, "  {}: <dropped>", address)?;
            }
        }
        Ok(())
    }
}
