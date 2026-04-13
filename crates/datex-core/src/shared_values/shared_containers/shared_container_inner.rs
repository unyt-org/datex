use crate::shared_values::pointer_address::{PointerAddress};
use crate::shared_values::shared_container::{ExternalSharedContainer};
use crate::shared_values::shared_container::shared_value_container::SharedValueContainer;
use crate::shared_values::shared_containers::SelfOwnedSharedContainer;

/// Wrapper containing either an [SelfOwnedSharedContainer] or an [ExternalSharedContainer].
#[derive(Debug)]
pub enum SharedContainerInner {
    EndpointOwned(SelfOwnedSharedContainer),
    External(ExternalSharedContainer),
}

impl SharedContainerInner {

    /// Get an immutable reference to the contained value
    pub fn value(&self) -> &SharedValueContainer {
        match self {
            SharedContainerInner::EndpointOwned(endpoint_owned) => endpoint_owned.value(),
            SharedContainerInner::External(external) => external.value(),
        }
    }

    /// Get a mutable reference to the contained value
    pub fn value_mut(&mut self) -> &mut SharedValueContainer {
        match self {
            SharedContainerInner::EndpointOwned(endpoint_owned) => endpoint_owned.value_mut(),
            SharedContainerInner::External(external) => external.value_mut(),
        }
    }

    /// Take the contained value out of the container, consuming the container in the process.
    pub fn take_value(self) -> SharedValueContainer {
        match self {
            SharedContainerInner::EndpointOwned(owned) => owned.take_value(),
            SharedContainerInner::External(referenced) => referenced.take_value(),
        }
    }

    /// Get the inner [PointerAddress].
    pub fn pointer_address(&self) -> PointerAddress {
        match self {
            SharedContainerInner::EndpointOwned(endpoint_owned) => PointerAddress::EndpointOwned(endpoint_owned.address().clone()),
            SharedContainerInner::External(external) => PointerAddress::External(external.address().clone()),
        }
    }
}