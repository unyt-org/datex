use alloc::rc::Rc;
use core::cell::RefCell;
use core::cell::{Ref, RefMut};
use core::fmt::Display;
use core::mem;
use crate::shared_values::errors::SharedValueCreationError;
use crate::shared_values::pointer_address::{SelfOwnedPointerAddress, ExternalPointerAddress};
use crate::shared_values::shared_containers::{SelfOwnedSharedContainer, ReferencedSharedContainer, SharedContainerInner, SharedContainerMutability};
use crate::shared_values::shared_containers::expose_rc_internal::ExposeRcInternal;
use crate::shared_values::shared_containers::base_shared_value_container::BaseSharedValueContainer;
use crate::types::r#type::Type;
use crate::types::structural_type_definition::StructuralTypeDefinition;
use crate::values::core_value::CoreValue;
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

/// Wrapper struct for an owned shared value (i.e. `shared X`)
/// It is guaranteed that the inner value is a [SharedContainerInner::EndpointOwned].
///
/// ([OwnedSharedContainer] implies [SharedContainerInner::EndpointOwned], but not vice versa,
/// since a [SharedContainerInner::EndpointOwned] can be wrapped in a [ReferencedSharedContainer])
///
/// When holding an [OwnedSharedContainer], it is guaranteed that the contained [SharedContainerInner] is
/// not moved and changed to [SharedContainerInner::External].
/// Only an [OwnedSharedContainer] can be moved to another endpoint or location.
#[derive(Debug)]
pub struct OwnedSharedContainer {
    /// It is guaranteed that the inner value is a [SharedContainerInner::EndpointOwned].
    inner: Rc<RefCell<SharedContainerInner>>,
}

impl OwnedSharedContainer {

    /// Creates a new owned container from an [SelfOwnedSharedContainer]
    pub fn new_from_self_owned_container(container: SelfOwnedSharedContainer) -> Self {
        OwnedSharedContainer {
            inner: Rc::new(RefCell::new(SharedContainerInner::EndpointOwned(container)))
        }
    }

    /// Tries to create a new [OwnedSharedContainer] with an initial [ValueContainer],
    /// an allowed [StructuralTypeDefinition], a [SharedContainerMutability] and an [SelfOwnedPointerAddress].
    ///
    /// If the allowed type is not a superset of the [ValueContainer]'s allowed type,
    /// an error is returned
    pub fn try_new(
        value_container: ValueContainer,
        allowed_type: StructuralTypeDefinition,
        mutability: SharedContainerMutability,
        address: SelfOwnedPointerAddress
    ) -> Result<Self, SharedValueCreationError> {
        Ok(OwnedSharedContainer::new_from_self_owned_container(
            SelfOwnedSharedContainer::new(
                BaseSharedValueContainer::try_new(
                    value_container,
                    allowed_type,
                    mutability,
                )?,
                address
            )
        ))
    }

    /// Creates a new [OwnedSharedContainer] with an initial [ValueContainer],
    /// a [SharedContainerMutability], and an [SelfOwnedPointerAddress].
    ///
    /// The allowed type is inferred from the value_container's allowed type.
    pub fn new_with_inferred_allowed_type(
        value_container: ValueContainer,
        mutability: SharedContainerMutability,
        address: SelfOwnedPointerAddress
    ) -> Self {
        OwnedSharedContainer::new_from_self_owned_container(
            SelfOwnedSharedContainer::new(
                BaseSharedValueContainer::new_with_inferred_allowed_type(
                    value_container,
                    mutability,
                ),
                address
            )
        )
    }

    pub fn inner(&self) -> Ref<SharedContainerInner> {
        self.inner.borrow()
    }
    pub fn inner_mut(&self) -> RefMut<SharedContainerInner> {
        self.inner.borrow_mut()
    }

    /// Gets a [Ref] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    pub fn base_shared_container(&self) -> Ref<BaseSharedValueContainer> {
        Ref::map(self.inner(), |inner| inner.base_shared_container())
    }

    /// Gets a [RefMut] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    pub fn base_shared_container_mut(&self) -> RefMut<BaseSharedValueContainer> {
        RefMut::map(self.inner_mut(), |inner| inner.base_shared_container_mut())
    }

    /// Gets a [Ref] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    pub fn value_container(&self) -> Ref<ValueContainer> {
        Ref::map(self.base_shared_container(), |base_shared_container| &base_shared_container.value_container)
    }

    /// Gets a [Ref] to the currently assigned allowed [StructuralTypeDefinition] of the shared container (not resolved recursively)
    pub fn allowed_type(&self) -> Ref<StructuralTypeDefinition> {
        Ref::map(self.base_shared_container(), |base_shared_container| &base_shared_container.allowed_type)
    }

    /// Gets a [RefMut] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    pub fn value_container_mut(&self) -> RefMut<ValueContainer> {
        RefMut::map(self.base_shared_container_mut(), |base_shared_container| &mut base_shared_container.value_container)
    }

    /// Get a [Ref] to the inner [SelfOwnedSharedContainer].
    /// It is guaranteed that the contained [SharedContainerInner] is always a [SharedContainerInner::EndpointOwned].
    pub fn as_self_owned_shared_container(&self) -> Ref<SelfOwnedSharedContainer> {
        Ref::map(self.inner(), |inner| match inner {
            SharedContainerInner::EndpointOwned(inner) => inner,
            _ => unreachable!("OwnedSharedContainer must contain an EndpointOwned inner value")
        })
    }

    /// Get a [RefMut] to the inner [SelfOwnedSharedContainer].
    /// It is guaranteed that the contained [SharedContainerInner] is always a [SharedContainerInner::EndpointOwned].
    pub fn as_self_owned_shared_container_mut(&self) -> RefMut<SelfOwnedSharedContainer> {
        RefMut::map(self.inner_mut(), |inner| match inner {
            SharedContainerInner::EndpointOwned(inner) => inner,
            _ => unreachable!("OwnedSharedContainer must contain an EndpointOwned inner value")
        })
    }

    /// Get a [Ref] to the inner [SelfOwnedPointerAddress].
    /// It is guaranteed that the pointer address is always a [SelfOwnedPointerAddress].
    pub fn pointer_address(&self) -> Ref<SelfOwnedPointerAddress> {
        Ref::map(self.as_self_owned_shared_container(), |inner| inner.address())
    }

    /// Get the [SharedContainerMutability] of the inner [SelfOwnedSharedContainer].
    pub fn container_mutability(&self) -> SharedContainerMutability {
        self.as_self_owned_shared_container().value().mutability.clone()
    }

    /// Creates a new immutable [ReferencedSharedContainer] pointing to the same inner value as this [OwnedSharedContainer].
    pub fn derive_immutable_reference(&self) -> ReferencedSharedContainer {
        ReferencedSharedContainer::new_immutable(
            self.inner.clone(),
        )
    }

    /// Tries to create a new mutable [ReferencedSharedContainer] pointing to the same inner value as this [OwnedSharedContainer].
    /// Returns an [Err] if the container itself is not mutable
    pub fn try_derive_mutable_reference(&self) -> Result<ReferencedSharedContainer, ()> {
        if self.container_mutability() == SharedContainerMutability::Mutable {
            return Err(());
        }

        // new_mutable_unchecked is safe to call here since we checked the container mutability before
        Ok(ReferencedSharedContainer::new_mutable_unchecked(
            self.inner.clone(),
        ))
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
        let mut inner = self.inner_mut();
        // replace previous with null value
        // FIXME: find a more efficient way to do this enum variant swap
        let previous =
            mem::replace(&mut *inner, SharedContainerInner::EndpointOwned(SelfOwnedSharedContainer::new(
                BaseSharedValueContainer {
                    value_container: ValueContainer::Local(Value {inner: CoreValue::Null, actual_type: Box::new(StructuralTypeDefinition::Unit) }),
                    allowed_type: StructuralTypeDefinition::Unit,
                    observers: Default::default(),
                    mutability: SharedContainerMutability::Immutable,
                },
                SelfOwnedPointerAddress::NULL,
            )));

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
        write!(f, "{}", self.as_self_owned_shared_container().value())
    }
}

impl ExposeRcInternal for OwnedSharedContainer {
    type Shared = SharedContainerInner;
    fn get_rc_internal(&self) -> &Rc<RefCell<Self::Shared>> {
        &self.inner
    }
}