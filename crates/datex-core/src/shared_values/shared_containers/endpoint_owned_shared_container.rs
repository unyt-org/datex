use crate::shared_values::pointer_address::{EndpointOwnedPointerAddress, ExternalPointerAddress};
use crate::shared_values::shared_container::shared_value_container::SharedValueContainer;
use crate::shared_values::shared_containers::ExternalSharedContainer;

/// A shared container with an endpoint-owned pointer address
#[derive(Debug)]
pub struct EndpointOwnedSharedContainer {
    pub value: SharedValueContainer,
    pub address: EndpointOwnedPointerAddress,
    // TODO #766: additional fields will probably be added later, e.g. previous owners
    // subscribers: Vec<(Endpoint, Permissions)>,
}

impl EndpointOwnedSharedContainer {
    /// Converts the [EndpointOwnedSharedContainer] into an [ExternalSharedContainer],
    /// setting the provided [ExternalPointerAddress]
    pub fn convert_to_external_container(
        self,
        external_address: ExternalPointerAddress,
    ) -> ExternalSharedContainer {
        ExternalSharedContainer {
            value: self.value,
            address: external_address,
        }
    }
}