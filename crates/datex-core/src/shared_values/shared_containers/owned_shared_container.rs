use alloc::rc::Rc;
use core::cell::RefCell;
use core::cell::{Ref, RefMut};
use crate::shared_values::pointer_address::EndpointOwnedPointerAddress;
use crate::shared_values::shared_container::SharedContainerInner;
use crate::shared_values::shared_containers::EndpointOwnedSharedContainer;

/// Wrapper struct for an owned shared value (i.e. `shared X`)
/// It is guaranteed that the inner value is a [SharedContainerInner::EndpointOwned].
///
/// ([OwnedSharedContainer] implies [SharedContainerInner::EndpointOwned], but not vice versa,
/// since a [SharedContainerInner::EndpointOwned] can be wrapped in a [ReferencedSharedContainer])
///
/// When holding a [OwnedSharedContainer], it is guaranteed that the contained [SharedContainerInner] is
/// not moved and changed to [SharedContainerInner::External].
/// Only a [OwnedSharedContainer] can be moved to another endpoint or location.
#[derive(Debug)]
pub struct OwnedSharedContainer {
    /// It is guaranteed that the inner value is a [SharedContainerInner::EndpointOwned].
    inner: Rc<RefCell<SharedContainerInner>>,
}

impl OwnedSharedContainer {
    /// Get a [Ref] to the inner [EndpointOwnedSharedContainer].
    /// It is guaranteed that the contained [SharedContainerInner] is always a [SharedContainerInner::EndpointOwned].
    pub fn as_endpoint_owned_shared_container(&self) -> Ref<EndpointOwnedSharedContainer> {
        Ref::map(self.inner.borrow(), |inner| match inner {
            SharedContainerInner::EndpointOwned(inner) => inner,
            _ => unreachable!("OwnedSharedContainer must contain an EndpointOwned inner value")
        })
    }

    /// Get a [RefMut] to the inner [EndpointOwnedSharedContainer].
    /// It is guaranteed that the contained [SharedContainerInner] is always a [SharedContainerInner::EndpointOwned].
    pub fn as_endpoint_owned_shared_container_mut(&self) -> RefMut<EndpointOwnedSharedContainer> {
        RefMut::map(self.inner.borrow_mut(), |inner| match inner {
            SharedContainerInner::EndpointOwned(inner) => inner,
            _ => unreachable!("OwnedSharedContainer must contain an EndpointOwned inner value")
        })
    }

    /// Get a [Ref] to the inner [EndpointOwnedPointerAddress].
    /// It is guaranteed that the pointer address is always a [EndpointOwnedPointerAddress].
    pub fn pointer_address(&self) -> Ref<EndpointOwnedPointerAddress> {
        Ref::map(self.as_endpoint_owned_shared_container(), |inner| &inner.address)
    }
}