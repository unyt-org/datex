use crate::shared_values::pointer_address::{ExternalPointerAddress};
use crate::shared_values::shared_containers::base_shared_value_container::BaseSharedValueContainer;

/// A shared container with an external pointer
#[derive(Debug)]
pub struct ExternalSharedContainer {
    value: BaseSharedValueContainer,
    /// Address of the external pointer, can be a remote or builtin pointer address
    address: ExternalPointerAddress,
}

impl ExternalSharedContainer {
    pub fn new(shared_value_container: BaseSharedValueContainer, address: ExternalPointerAddress) -> Self {
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
