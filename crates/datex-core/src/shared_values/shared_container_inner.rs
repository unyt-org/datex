use core::mem;
use crate::shared_values::{ExternalSharedContainer, SelfOwnedSharedContainer, base_shared_value_container::BaseSharedValueContainer, pointer_address::PointerAddress, SelfOwnedPointerAddress};

/// Wrapper containing either an [SelfOwnedSharedContainer] or an [ExternalSharedContainer].
#[derive(Debug)]
pub enum SharedContainerInner {
    EndpointOwned(SelfOwnedSharedContainer),
    External(ExternalSharedContainer),
}

impl SharedContainerInner {
    /// Get an immutable reference to the contained value
    pub fn base_shared_container(&self) -> &BaseSharedValueContainer {
        match self {
            SharedContainerInner::EndpointOwned(endpoint_owned) => {
                endpoint_owned.value()
            }
            SharedContainerInner::External(external) => external.value(),
        }
    }

    /// Get a mutable reference to the contained value
    pub fn base_shared_container_mut(
        &mut self,
    ) -> &mut BaseSharedValueContainer {
        match self {
            SharedContainerInner::EndpointOwned(endpoint_owned) => {
                endpoint_owned.value_mut()
            }
            SharedContainerInner::External(external) => external.value_mut(),
        }
    }

    /// Take the contained value out of the container, consuming the container in the process.
    pub fn take_base_shared_container(self) -> BaseSharedValueContainer {
        match self {
            SharedContainerInner::EndpointOwned(owned) => owned.take_value(),
            SharedContainerInner::External(referenced) => {
                referenced.take_value()
            }
        }
    }

    /// Get the inner [PointerAddress].
    pub fn pointer_address(&self) -> PointerAddress {
        match self {
            SharedContainerInner::EndpointOwned(endpoint_owned) => {
                PointerAddress::SelfOwned(endpoint_owned.address().clone())
            }
            SharedContainerInner::External(external) => {
                PointerAddress::Remote(external.address().clone())
            }
        }
    }

    /// Change the inner [PointerAddress] to a new one, potentially changing the type of the container.
    /// # Safety
    /// The caller must ensure that the new [PointerAddress] is not already used by another shared container
    pub unsafe fn change_address(
        &mut self, 
        new_address: PointerAddress
    ) {
        let previous = mem::replace(
            &mut *self,
            SharedContainerInner::EndpointOwned(unsafe {
                SelfOwnedSharedContainer::new_with_address(
                    BaseSharedValueContainer::null(),
                    SelfOwnedPointerAddress([0; 5]),
                )
            }),
        );

        // TODO: handle subscriber switch etc
        match new_address {
            PointerAddress::SelfOwned(new_self_owned_address) => {
                *self = SharedContainerInner::EndpointOwned(unsafe {
                    SelfOwnedSharedContainer::new_with_address(
                        previous.take_base_shared_container(),
                        new_self_owned_address,
                    )
                });
            }
            PointerAddress::Remote(new_remote_address) => {
                *self = SharedContainerInner::External(unsafe {
                    ExternalSharedContainer::new(
                        previous.take_base_shared_container(),
                        new_remote_address,
                    )
                });
            }
        }
    }
}
