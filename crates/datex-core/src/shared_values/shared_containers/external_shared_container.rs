use crate::shared_values::pointer_address::{ExternalPointerAddress};
use crate::shared_values::shared_container::shared_value_container::SharedValueContainer;

/// A shared container with an external pointer
#[derive(Debug)]
pub struct ExternalSharedContainer {
    value: SharedValueContainer,
    /// Address of the external pointer, can be a remote or builtin pointer address
    address: ExternalPointerAddress,
}

impl ExternalSharedContainer {
    pub fn new(shared_value_container: SharedValueContainer, address: ExternalPointerAddress) -> Self {
        ExternalSharedContainer {
            value: shared_value_container,
            address,
        }
    }

    pub fn value(&self) -> &SharedValueContainer {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut SharedValueContainer {
        &mut self.value
    }

    pub fn take_value(self) -> SharedValueContainer {
        self.value
    }

    pub fn address(&self) -> &ExternalPointerAddress {
        &self.address
    }
}
