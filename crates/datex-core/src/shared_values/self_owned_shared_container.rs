use crate::{
    runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
    shared_values::{
        ExternalSharedContainer, RemotePointerAddress, SelfOwnedPointerAddress,
        base_shared_value_container::BaseSharedValueContainer,
    },
};

/// A shared container with a pointer address owned by the local endpoint
#[derive(Debug)]
pub struct SelfOwnedSharedContainer {
    value: BaseSharedValueContainer,
    address: SelfOwnedPointerAddress,
    // TODO #766: additional fields will probably be added later, e.g. previous owners
    // subscribers: Vec<(Endpoint, Permissions)>,
}

impl SelfOwnedSharedContainer {
    /// Creates a new [SelfOwnedSharedContainer]
    pub fn new(
        shared_value_container: BaseSharedValueContainer,
        self_owned_pointer_address_provider: &mut SelfOwnedPointerAddressProvider,
    ) -> Self {
        SelfOwnedSharedContainer {
            value: shared_value_container,
            address: self_owned_pointer_address_provider
                .get_new_self_owned_address(),
        }
    }

    /// Creates a new [SelfOwnedSharedContainer]
    pub unsafe fn new_with_address(
        shared_value_container: BaseSharedValueContainer,
        address: SelfOwnedPointerAddress,
    ) -> Self {
        SelfOwnedSharedContainer {
            value: shared_value_container,
            address,
        }
    }

    pub fn value(&self) -> &BaseSharedValueContainer {
        &self.value
    }

    pub fn take_value(self) -> BaseSharedValueContainer {
        self.value
    }

    pub fn value_mut(&mut self) -> &mut BaseSharedValueContainer {
        &mut self.value
    }

    pub fn address(&self) -> &SelfOwnedPointerAddress {
        &self.address
    }

    /// Converts the [SelfOwnedSharedContainer] into an [ExternalSharedContainer],
    /// setting the provided [RemotePointerAddress]
    /// # Safety
    /// The caller must ensure that the [RemotePointerAddress] does not yet exist in the [SharedReferencesCache]
    /// # Safety
    /// TODO: handle subscriber transfer somewhere
    pub unsafe fn convert_to_external_container(
        self,
        remote_address: RemotePointerAddress,
    ) -> ExternalSharedContainer {
        unsafe { ExternalSharedContainer::new(self.value, remote_address) }
    }
}
