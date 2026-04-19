use crate::{
    shared_values::{
        errors::SharedValueCreationError,
        pointer_address::{ExternalPointerAddress, SelfOwnedPointerAddress},
        shared_containers::{
            ReferencedSharedContainer, SelfOwnedSharedContainer,
            SharedContainerInner, SharedContainerMutability,
            base_shared_value_container::BaseSharedValueContainer,
            internal_traits::_ExposeRcInternal,
        },
    },
    types::type_definition::TypeDefinition,
    values::{
        core_value::CoreValue, value::Value, value_container::ValueContainer,
    },
};
use alloc::{boxed::Box, rc::Rc};
use core::{
    cell::{Ref, RefCell, RefMut},
    fmt::Display,
    mem,
};
use serde::Serialize;
use crate::runtime::memory::Memory;
use crate::runtime::pointer_address_provider::SelfOwnedPointerAddressProvider;
use crate::serde::Deserialize;
use crate::shared_values::pointer_address::PointerAddress;
use crate::types::literal_type_definition::LiteralTypeDefinition;
use crate::types::r#type::Type;

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
    pub fn new_from_self_owned_container(
        container: SelfOwnedSharedContainer,
    ) -> Self {
        OwnedSharedContainer {
            inner: Rc::new(RefCell::new(SharedContainerInner::EndpointOwned(
                container,
            ))),
        }
    }

    /// Tries to create a new [OwnedSharedContainer] with an initial [ValueContainer],
    /// an allowed [Type], a [SharedContainerMutability] and an [SelfOwnedPointerAddress].
    ///
    /// If the allowed type is not a superset of the [ValueContainer]'s allowed type,
    /// an error is returned
    pub fn try_new(
        value_container: ValueContainer,
        allowed_type: Type,
        mutability: SharedContainerMutability,
        address: SelfOwnedPointerAddress,
    ) -> Result<Self, SharedValueCreationError> {
        Ok(OwnedSharedContainer::new_from_self_owned_container(
            SelfOwnedSharedContainer::new(
                BaseSharedValueContainer::try_new(
                    value_container,
                    allowed_type,
                    mutability,
                )?,
                address,
            ),
        ))
    }

    /// Creates a new [OwnedSharedContainer] with an initial [ValueContainer],
    /// a [SharedContainerMutability], and an [SelfOwnedPointerAddress].
    ///
    /// The allowed type is inferred from the value_container's allowed type.
    pub fn new_with_inferred_allowed_type(
        value_container: ValueContainer,
        mutability: SharedContainerMutability,
        address_provider: &mut SelfOwnedPointerAddressProvider,
        memory: &Memory,
    ) -> Self {
        // Note: address provider guarantees new unique address
        unsafe {
            Self::new_with_inferred_allowed_type_unsafe(
                value_container,
                mutability,
                address_provider.get_new_self_owned_address(),
                memory,
            )
        }
    }


    /// Creates a new [OwnedSharedContainer] with an initial [ValueContainer],
    /// a [SharedContainerMutability], and an [SelfOwnedPointerAddress].
    ///
    /// The allowed type is inferred from the value_container's allowed type.
    ///
    /// The caller must ensure that the address is not used anywhere else.
    pub unsafe fn new_with_inferred_allowed_type_unsafe(
        value_container: ValueContainer,
        mutability: SharedContainerMutability,
        address: SelfOwnedPointerAddress,
        memory: &Memory,
    ) -> Self {
        OwnedSharedContainer::new_from_self_owned_container(
            SelfOwnedSharedContainer::new(
                BaseSharedValueContainer::new_with_inferred_allowed_type(
                    value_container,
                    mutability,
                    memory
                ),
                address,
            ),
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
    pub fn base_shared_container_mut(
        &self,
    ) -> RefMut<BaseSharedValueContainer> {
        RefMut::map(self.inner_mut(), |inner| inner.base_shared_container_mut())
    }

    /// Gets a [Ref] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    pub fn value_container(&self) -> Ref<ValueContainer> {
        Ref::map(self.base_shared_container(), |base_shared_container| {
            &base_shared_container.value_container
        })
    }

    /// Gets a [Ref] to the currently assigned allowed [Type] of the shared container (not resolved recursively)
    pub fn allowed_type(&self) -> Ref<Type> {
        Ref::map(self.base_shared_container(), |base_shared_container| {
            &base_shared_container.allowed_type
        })
    }

    /// Gets a [RefMut] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    pub fn value_container_mut(&self) -> RefMut<ValueContainer> {
        RefMut::map(self.base_shared_container_mut(), |base_shared_container| {
            &mut base_shared_container.value_container
        })
    }

    /// Calls the provided callback with a mut reference to the recursively collapsed inner value of the shared container
    pub fn with_collapsed_value_mut<R>(
        &self,
        f: impl FnOnce(&mut Value) -> R,
    ) -> R {
        self.inner_mut().base_shared_container_mut().with_collapsed_value_mut(f)
    }

    /// Calls the provided callback with a reference to the recursively collapsed inner value of the shared container
    pub fn with_collapsed_value<R>(&self, f: impl FnOnce(&Value) -> R) -> R {
        self.inner().base_shared_container().with_collapsed_value(f)
    }

    /// Get a [Ref] to the inner [SelfOwnedSharedContainer].
    /// It is guaranteed that the contained [SharedContainerInner] is always a [SharedContainerInner::EndpointOwned].
    pub fn as_self_owned_shared_container(
        &self,
    ) -> Ref<SelfOwnedSharedContainer> {
        Ref::map(self.inner(), |inner| match inner {
            SharedContainerInner::EndpointOwned(inner) => inner,
            _ => unreachable!(
                "OwnedSharedContainer must contain an EndpointOwned inner value"
            ),
        })
    }

    /// Get a [RefMut] to the inner [SelfOwnedSharedContainer].
    /// It is guaranteed that the contained [SharedContainerInner] is always a [SharedContainerInner::EndpointOwned].
    pub fn as_self_owned_shared_container_mut(
        &self,
    ) -> RefMut<SelfOwnedSharedContainer> {
        RefMut::map(self.inner_mut(), |inner| match inner {
            SharedContainerInner::EndpointOwned(inner) => inner,
            _ => unreachable!(
                "OwnedSharedContainer must contain an EndpointOwned inner value"
            ),
        })
    }

    /// Get a [Ref] to the inner [SelfOwnedPointerAddress].
    /// It is guaranteed that the pointer address is always a [SelfOwnedPointerAddress].
    pub fn pointer_address(&self) -> Ref<SelfOwnedPointerAddress> {
        Ref::map(self.as_self_owned_shared_container(), |inner| {
            inner.address()
        })
    }

    /// Get the [SharedContainerMutability] of the inner [SelfOwnedSharedContainer].
    pub fn container_mutability(&self) -> SharedContainerMutability {
        self.as_self_owned_shared_container()
            .value()
            .mutability
            .clone()
    }

    /// Creates a new immutable [ReferencedSharedContainer] pointing to the same inner value as this [OwnedSharedContainer].
    pub fn derive_immutable_reference(&self) -> ReferencedSharedContainer {
        ReferencedSharedContainer::new_immutable(self.inner.clone())
    }

    /// Tries to create a new mutable [ReferencedSharedContainer] pointing to the same inner value as this [OwnedSharedContainer].
    /// Returns an [Err] if the container itself is not mutable
    pub fn try_derive_mutable_reference(
        &self,
    ) -> Result<ReferencedSharedContainer, ()> {
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
    ///
    /// The caller must ensure that the [ExternalPointerAddress] does not yet exist in the [Memory]
    pub unsafe fn move_to_external(self, external_address: ExternalPointerAddress, memory: &Memory) {
        let mut inner = self.inner_mut();
        // replace previous with null value
        // FIXME: find a more efficient way to do this enum variant swap
        let previous = mem::replace(
            &mut *inner,
            SharedContainerInner::EndpointOwned(SelfOwnedSharedContainer::new(
                BaseSharedValueContainer {
                    value_container: ValueContainer::Local(Value {
                        inner: CoreValue::Null,
                        custom_type: None,
                    }),
                    allowed_type: Type::from(LiteralTypeDefinition::Unit),
                    observers: Default::default(),
                    mutability: SharedContainerMutability::Immutable,
                },
                SelfOwnedPointerAddress {address: [0; 5]},
            )),
        );

        *inner = match previous {
            SharedContainerInner::EndpointOwned(owned) => {
                SharedContainerInner::External(
                  unsafe { owned.convert_to_external_container(external_address, memory) },
                )
            }
            _ => unreachable!(
                "OwnedSharedContainer must contain an EndpointOwned inner value"
            ),
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

impl _ExposeRcInternal for OwnedSharedContainer {
    type Shared = SharedContainerInner;
    fn get_rc_internal(&self) -> &Rc<RefCell<Self::Shared>> {
        &self.inner
    }
}
