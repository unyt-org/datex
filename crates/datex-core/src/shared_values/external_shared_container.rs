use crate::{
    runtime::memory::Memory,
    shared_values::{
        PointerAddress,
        base_shared_value_container::BaseSharedValueContainer,
    },
};
use crate::shared_values::RemotePointerAddress;

/// A shared container with an external pointer
#[derive(Debug)]
pub struct ExternalSharedContainer {
    value: BaseSharedValueContainer,
    /// Address of the remote pointer, can be a remote or builtin pointer address
    address: RemotePointerAddress,
}

impl ExternalSharedContainer {
    /// Create a new [ExternalSharedContainer] with a given [RemotePointerAddress].
    /// # Safety
    /// The caller must ensure that the [RemotePointerAddress] does not yet exist in the [Memory]
    pub unsafe fn create_external_shared_container(
        shared_value_container: BaseSharedValueContainer,
        address: RemotePointerAddress,
        memory: &Memory,
    ) -> ExternalSharedContainer {
        if memory.has_reference(&PointerAddress::Remote(address.clone())) {
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

    pub fn address(&self) -> &RemotePointerAddress {
        &self.address
    }
}
