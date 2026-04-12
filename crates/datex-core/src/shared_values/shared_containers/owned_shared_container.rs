use alloc::rc::Rc;
use core::cell::RefCell;
use core::cell::{Ref, RefMut};
use core::fmt::Display;
use core::mem;
use crate::shared_values::pointer::ExternalPointer;
use crate::shared_values::pointer_address::{EndpointOwnedPointerAddress, ExternalPointerAddress};
use crate::shared_values::shared_container::{OwnedOrReferencedSharedContainer, ReferenceMutability, SharedContainerInner, SharedContainerMutability};
use crate::shared_values::shared_containers::{EndpointOwnedSharedContainer, ReferencedSharedContainer};
use crate::shared_values::shared_containers::shared_value_container::SharedValueContainer;
use crate::types::definition::TypeDefinition;
use crate::values::core_value::CoreValue;
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

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

    /// Get a reference to the inner [Rc<RefCell<SharedContainerInner>>]
    pub fn inner_rc(&self) -> &Rc<RefCell<SharedContainerInner>> {
        &self.inner
    }

    pub fn as_inner(&self) -> Ref<SharedContainerInner> {
        self.inner.borrow()
    }
    pub fn as_inner_mut(&self) -> RefMut<SharedContainerInner> {
        self.inner.borrow_mut()
    }

    /// Get a [Ref] to the inner [EndpointOwnedSharedContainer].
    /// It is guaranteed that the contained [SharedContainerInner] is always a [SharedContainerInner::EndpointOwned].
    pub fn as_endpoint_owned_shared_container(&self) -> Ref<EndpointOwnedSharedContainer> {
        Ref::map(self.as_inner(), |inner| match inner {
            SharedContainerInner::EndpointOwned(inner) => inner,
            _ => unreachable!("OwnedSharedContainer must contain an EndpointOwned inner value")
        })
    }

    /// Get a [RefMut] to the inner [EndpointOwnedSharedContainer].
    /// It is guaranteed that the contained [SharedContainerInner] is always a [SharedContainerInner::EndpointOwned].
    pub fn as_endpoint_owned_shared_container_mut(&self) -> RefMut<EndpointOwnedSharedContainer> {
        RefMut::map(self.as_inner_mut(), |inner| match inner {
            SharedContainerInner::EndpointOwned(inner) => inner,
            _ => unreachable!("OwnedSharedContainer must contain an EndpointOwned inner value")
        })
    }

    /// Get a [Ref] to the inner [EndpointOwnedPointerAddress].
    /// It is guaranteed that the pointer address is always a [EndpointOwnedPointerAddress].
    pub fn pointer_address(&self) -> Ref<EndpointOwnedPointerAddress> {
        Ref::map(self.as_endpoint_owned_shared_container(), |inner| &inner.address)
    }

    /// Get the [SharedContainerMutability] of the inner [EndpointOwnedSharedContainer].
    pub fn container_mutability(&self) -> SharedContainerMutability {
        self.as_endpoint_owned_shared_container().value.mutability.clone()
    }

    /// Creates a new immutable [ReferencedSharedContainer] pointing to the same inner value as this [OwnedSharedContainer].
    pub fn derive_immutable_reference(&self) -> ReferencedSharedContainer {
        ReferencedSharedContainer {
            inner: self.inner.clone(),
            reference_mutability: ReferenceMutability::Immutable,
        }
    }

    /// Tries to create a new mutable [ReferencedSharedContainer] pointing to the same inner value as this [OwnedSharedContainer].
    /// Returns an [Err] if the container itself is not mutable
    pub fn try_derive_mutable_reference(&self) -> Result<ReferencedSharedContainer, ()> {
        if self.container_mutability() == SharedContainerMutability::Mutable {
            return Err(());
        }

        Ok(ReferencedSharedContainer {
            inner: self.inner.clone(),
            reference_mutability: ReferenceMutability::Mutable,
        })
    }

    /// Clones the shared container as a mutable reference if possible, otherwise as an immutable reference
    pub fn derive_with_max_mutability(&self) -> ReferencedSharedContainer {
        self.try_derive_mutable_reference()
            .unwrap_or_else(|_| self.derive_immutable_reference())
    }


    /// Moves an owned shared container by converting it to a [ReferencedSharedContainer] with an [ExternalPointer] pointing to the given remote address.
    /// Drops the original owned shared container
    pub fn move_to_external(
        self,
        external_address: ExternalPointerAddress,
    ) {
        let mut inner = self.as_inner_mut();
        // replace previous with null value 
        // FIXME: find a more efficient way to do this enum variant swap
        let previous =
            mem::replace(&mut *inner, SharedContainerInner::EndpointOwned(EndpointOwnedSharedContainer {
                value: SharedValueContainer {
                    value_container: ValueContainer::Local(Value {inner: CoreValue::Null, actual_type: Box::new(TypeDefinition::Unit) }),
                    allowed_type: TypeDefinition::Unit,
                    observers: Default::default(),
                    mutability: SharedContainerMutability::Immutable,
                },
                address: EndpointOwnedPointerAddress::NULL,
            }));

        *inner = match previous {
            SharedContainerInner::EndpointOwned(owned) => 
                SharedContainerInner::External(owned.convert_to_external_container(external_address)),
            _ => unreachable!("OwnedSharedContainer must contain an EndpointOwned inner value"),
        };
    }


    /// Checks if the owned container can be mutated by the local endpoint
    pub(crate) fn can_mutate(&self) -> bool {
        self.container_mutability() == SharedContainerMutability::Mutable
    }
}

impl Display for OwnedSharedContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_endpoint_owned_shared_container().value)
    }
}