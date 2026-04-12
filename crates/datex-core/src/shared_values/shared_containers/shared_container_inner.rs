use std::cell::Ref;
use crate::shared_values::pointer_address::{EndpointOwnedPointerAddress, PointerAddress};
use crate::shared_values::shared_container::ExternalSharedContainer;
use crate::shared_values::shared_container::shared_value_container::SharedValueContainer;
use crate::shared_values::shared_containers::EndpointOwnedSharedContainer;

/// Wrapper containing either an endpoint-owned shared container or an external shared container
#[derive(Debug)]
pub enum SharedContainerInner {
    EndpointOwned(EndpointOwnedSharedContainer),
    External(ExternalSharedContainer),
}

impl SharedContainerInner {

    /// Get an immutable reference to the contained value
    pub fn value(&self) -> &SharedValueContainer {
        match self {
            SharedContainerInner::EndpointOwned(endpoint_owned) => &endpoint_owned.value,
            SharedContainerInner::External(external) => &external.value,
        }
    }

    /// Get a mutable reference to the contained value
    pub fn value_mut(&mut self) -> &mut SharedValueContainer {
        match self {
            SharedContainerInner::EndpointOwned(endpoint_owned) => &mut endpoint_owned.value,
            SharedContainerInner::External(external) => &mut external.value,
        }
    }

    /// Take the contained value out of the container, consuming the container in the process.
    pub fn take_value(self) -> SharedValueContainer {
        match self {
            SharedContainerInner::EndpointOwned(owned) => owned.value,
            SharedContainerInner::External(referenced) => referenced.value,
        }
    }

    /// Get the inner [PointerAddress].
    pub fn pointer_address(&self) -> PointerAddress {
        match self {
            SharedContainerInner::EndpointOwned(endpoint_owned) => PointerAddress::EndpointOwned(endpoint_owned.address.clone()),
            SharedContainerInner::External(external) => PointerAddress::External(external.address.clone()),
        }
    }

}