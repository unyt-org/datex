use crate::shared_values::pointer_address::{SelfOwnedPointerAddress, ExternalPointerAddress};
use crate::shared_values::shared_container::shared_value_container::SharedValueContainer;
use crate::shared_values::shared_containers::ExternalSharedContainer;

/// A shared container with a pointer address owned by the local endpoint
#[derive(Debug)]
pub struct SelfOwnedSharedContainer {
    value: SharedValueContainer,
    address: SelfOwnedPointerAddress,
    // TODO #766: additional fields will probably be added later, e.g. previous owners
    // subscribers: Vec<(Endpoint, Permissions)>,
}

impl SelfOwnedSharedContainer {
    
    /// Creates a new [SelfOwnedSharedContainer]
    pub fn new(shared_value_container: SharedValueContainer, address: SelfOwnedPointerAddress) -> Self {
        SelfOwnedSharedContainer {
            value: shared_value_container,
            address,
        }
    }
    
    pub fn value(&self) -> &SharedValueContainer {
        &self.value
    }

    pub fn take_value(self) -> SharedValueContainer {
        self.value
    }
    
    pub fn value_mut(&mut self) -> &mut SharedValueContainer {
        &mut self.value
    }
    
    pub fn address(&self) -> &SelfOwnedPointerAddress {
        &self.address
    }
    
    /// Converts the [SelfOwnedSharedContainer] into an [ExternalSharedContainer],
    /// setting the provided [ExternalPointerAddress]
    /// TODO: handle subscriber transfer somewhere
    pub fn convert_to_external_container(
        self,
        external_address: ExternalPointerAddress,
    ) -> ExternalSharedContainer {
        ExternalSharedContainer::new(
            self.value,
            external_address,
        )
    }
}