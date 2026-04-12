use crate::shared_values::pointer_address::EndpointOwnedPointerAddress;
use crate::shared_values::shared_container::shared_value_container::SharedValueContainer;

/// A shared container with an endpoint-owned pointer address
#[derive(Debug)]
pub struct EndpointOwnedSharedContainer {
    pub value: SharedValueContainer,
    pub address: EndpointOwnedPointerAddress,
    // TODO #766: additional fields will probably be added later, e.g. previous owners
    // subscribers: Vec<(Endpoint, Permissions)>,
}