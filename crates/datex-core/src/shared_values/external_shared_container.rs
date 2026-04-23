use crate::{
    runtime::memory::Memory,
    shared_values::{
        ExternalPointerAddress, PointerAddress,
        base_shared_value_container::BaseSharedValueContainer,
    },
};

/// A shared container with an external pointer
#[derive(Debug)]
pub struct ExternalSharedContainer {
    value: BaseSharedValueContainer,
    /// Address of the external pointer, can be a remote or builtin pointer address
    address: ExternalPointerAddress,
}

impl ExternalSharedContainer {
    /// Create a new [ExternalSharedContainer] with a given [ExternalPointerAddress].
    /// The caller must ensure that the [ExternalPointerAddress] does not yet exist in the [Memory]
    pub unsafe fn create_external_shared_container(
        shared_value_container: BaseSharedValueContainer,
        address: ExternalPointerAddress,
        memory: &Memory,
    ) -> ExternalSharedContainer {
        if memory.has_reference(&PointerAddress::External(address.clone())) {
            panic!(
                "Cannot create ExternalSharedContainer with address that already exists in memory"
            );
        }

        ExternalSharedContainer {
            value: shared_value_container,
            address,
        }
    }
    pub fn value(&self) -> &BaseSharedValueContainer {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut BaseSharedValueContainer {
        &mut self.value
    }

    pub fn take_value(self) -> BaseSharedValueContainer {
        self.value
    }

    pub fn address(&self) -> &ExternalPointerAddress {
        &self.address
    }
}
