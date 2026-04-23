use crate::{
    collections::HashMap,
    libs::{core::CoreLibrary, library::Library},
    shared_values::{
        pointer_address::PointerAddress,
        shared_containers::ReferencedSharedContainer,
    },
};

#[derive(Debug)]
pub struct Memory {
    /// Shared values that are actively referenced or owned somewhere
    /// in the runtime or on remote endpoints
    values: HashMap<PointerAddress, ReferencedSharedContainer>,
}

impl Memory {
    /// Creates a new, Memory instance with the core library loaded.
    pub fn new() -> Memory {
        let mut memory = Memory {
            values: HashMap::new(),
        };
        // load core library
        // Note: safe because memory is newly initialized without core lib
        unsafe {
            CoreLibrary::load(&mut memory);
        }

        memory
    }

    /// Registers a referenced shared container in memory. If the reference has no PointerAddress, a new local one is generated.
    /// If the reference is already registered (has a PointerAddress), the existing address is returned and no new registration is done.
    /// Owned shared containers shall not be registered in memory.
    /// Returns the PointerAddress of the registered reference.
    pub fn register_referenced_shared_container(
        &mut self,
        container: &ReferencedSharedContainer,
    ) {
        let pointer_address = container.pointer_address();
        // check if reference is already registered (if it has an address, we assume it is registered)
        self.values
            .entry(pointer_address)
            .or_insert_with(|| container.clone());
    }

    /// Returns a reference stored at the given PointerAddress, if it exists.
    pub fn get_reference(
        &self,
        pointer_address: &PointerAddress,
    ) -> Option<&ReferencedSharedContainer> {
        self.values.get(pointer_address)
    }

    /// Checks if a reference with the given PointerAddress exists in memory.
    pub fn has_reference(&self, pointer_address: &PointerAddress) -> bool {
        self.values.contains_key(pointer_address)
    }

    pub fn values(
        &self,
    ) -> &HashMap<PointerAddress, ReferencedSharedContainer> {
        &self.values
    }
}
