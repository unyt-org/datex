use crate::{
    runtime::memory::Memory,
    shared_values::{
        ExternalPointerAddress, ExternalSharedContainer,
        SelfOwnedPointerAddress,
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
    /// setting the provided [ExternalPointerAddress]
    ///
    /// The caller must ensure that the [ExternalPointerAddress] does not yet exist in the [Memory]
    ///
    /// TODO: handle subscriber transfer somewhere
    pub unsafe fn convert_to_external_container(
        self,
        external_address: ExternalPointerAddress,
        memory: &Memory,
    ) -> ExternalSharedContainer {
        unsafe {
            ExternalSharedContainer::create_external_shared_container(
                self.value,
                external_address,
                memory,
            )
        }
    }
}
