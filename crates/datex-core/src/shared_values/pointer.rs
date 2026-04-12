use crate::shared_values::pointer_address::{
    EndpointOwnedPointerAddress, PointerAddress, ExternalPointerAddress,
};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EndpointOwnedPointer {
    /// Address of the owned pointer, must be a local pointer address
    address: EndpointOwnedPointerAddress,
    // TODO #766: additional fields will probably be added later, e.g. previous owners
    // subscribers: Vec<(Endpoint, Permissions)>,
}

impl EndpointOwnedPointer {
    pub const NULL: EndpointOwnedPointer = EndpointOwnedPointer {
        address: EndpointOwnedPointerAddress::NULL,
    };

    pub fn new(address: EndpointOwnedPointerAddress) -> Self {
        EndpointOwnedPointer { address }
    }

    pub fn address(&self) -> &EndpointOwnedPointerAddress {
        &self.address
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalPointer {
    /// Address of the borrowed pointer, can be an internal or remote pointer address
    address: ExternalPointerAddress,
}

impl ExternalPointer {
    pub fn new(address: ExternalPointerAddress) -> Self {
        ExternalPointer { address }
    }
    pub fn address(&self) -> &ExternalPointerAddress {
        &self.address
    }
}
