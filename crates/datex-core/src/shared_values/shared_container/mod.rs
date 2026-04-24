use crate::{
    runtime::{
        execution::ExecutionError, memory::Memory,
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::{
        OwnedSharedContainer, PointerAddress, ReferencedSharedContainer,
        SelfOwnedPointerAddress, SharedContainerInner,
        SharedContainerMutability, SharedContainerOwnership,
        base_shared_value_container::BaseSharedValueContainer,
        errors::{
            AccessError, UnexpectedImmutableReferenceError,
            UnexpectedSharedContainerOwnershipError,
        },
        internal_traits::_ExposeRcInternal,
        observers::{Observer, ObserverError, ObserverId},
    },
    traits::{
        apply::Apply, identity::Identity, structural_eq::StructuralEq,
        value_eq::ValueEq,
    },
    types::r#type::Type,
    values::{
        value::Value,
        value_container::{ValueContainer, value_key::BorrowedValueKey},
    },
};
use alloc::rc::Rc;
use core::{
    cell::{Ref, RefCell, RefMut},
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};
pub mod apply;
pub mod serde_dif;
/// Top-level wrapper for any owned or referenced shared container,
/// which can either be an owned shared container or a reference to a shared container.
#[derive(Debug)]
pub enum SharedContainer {
    /// An owned shared container (`shared X`). This is always points to a [SharedContainerInner::EndpointOwned]
    Owned(OwnedSharedContainer),
    /// A referenced shared container (`'shared X` or `'mut shared X`).
    /// This can point to either a [SharedContainerInner::EndpointOwned] or a [SharedContainerInner::External]
    Referenced(ReferencedSharedContainer),
}

impl SharedContainer {
    /// Creates a new owned [SharedContainer] with an initial [ValueContainer],
    /// a [SharedContainerMutability], and a [SelfOwnedPointerAddressProvider].
    ///
    /// The allowed type is inferred from the value_container's allowed type.
    pub fn new_owned_with_inferred_allowed_type<T: Into<ValueContainer>>(
        value_container: T,
        mutability: SharedContainerMutability,
        address_provider: &mut SelfOwnedPointerAddressProvider,
        memory: &Memory,
    ) -> Self {
        SharedContainer::Owned(
            OwnedSharedContainer::new_with_inferred_allowed_type(
                value_container.into(),
                mutability,
                address_provider,
                memory,
            ),
        )
    }

    /// Creates a new owned [SharedContainer] with an initial [ValueContainer],
    /// a [SharedContainerMutability], and a [SelfOwnedPointerAddress].
    ///
    /// The allowed type is inferred from the value_container's allowed type.
    ///
    /// The caller must ensure that the address is not used anywhere else.
    pub unsafe fn new_owned_with_inferred_allowed_type_unsafe<
        T: Into<ValueContainer>,
    >(
        value_container: T,
        mutability: SharedContainerMutability,
        address: SelfOwnedPointerAddress,
        memory: &Memory,
    ) -> Self {
        SharedContainer::Owned(unsafe {
            OwnedSharedContainer::new_with_inferred_allowed_type_unsafe(
                value_container.into(),
                mutability,
                address,
                memory,
            )
        })
    }

    pub fn inner(&self) -> Ref<'_, SharedContainerInner> {
        match self {
            SharedContainer::Owned(owned) => owned.inner(),
            SharedContainer::Referenced(referenced) => referenced.inner(),
        }
    }

    pub fn inner_mut(&self) -> RefMut<'_, SharedContainerInner> {
        match self {
            SharedContainer::Owned(owned) => owned.inner_mut(),
            SharedContainer::Referenced(referenced) => referenced.inner_mut(),
        }
    }

    /// Gets a [Ref] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    pub fn base_shared_container(&self) -> Ref<'_, BaseSharedValueContainer> {
        match self {
            SharedContainer::Owned(owned) => owned.base_shared_container(),
            SharedContainer::Referenced(referenced) => {
                referenced.base_shared_container()
            }
        }
    }

    /// Adds an observer to this shared container that will be notified on value changes.
    pub fn observe(
        &self,
        observer: Observer,
    ) -> Result<ObserverId, ObserverError> {
        self.base_shared_container_mut().observe(observer)
    }

    pub fn unobserve(
        &self,
        observer_id: ObserverId,
    ) -> Result<(), ObserverError> {
        self.base_shared_container_mut().unobserve(observer_id)
    }

    /// Gets a [RefMut] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    pub fn base_shared_container_mut(
        &self,
    ) -> RefMut<'_, BaseSharedValueContainer> {
        match self {
            SharedContainer::Owned(owned) => owned.base_shared_container_mut(),
            SharedContainer::Referenced(referenced) => {
                referenced.base_shared_container_mut()
            }
        }
    }

    /// Sets the currently assigned [ValueContainer] of the shared container to a new value container.
    /// Returns the [ValueContainer] as an error if it could not be assigned
    pub fn try_set_value_container(
        &self,
        new_value_container: ValueContainer,
    ) -> Result<(), ValueContainer> {
        self.base_shared_container_mut()
            .try_set_value_container(new_value_container)
    }

    /// Gets a [Ref] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    pub fn value_container(&self) -> Ref<'_, ValueContainer> {
        match self {
            SharedContainer::Owned(owned) => owned.value_container(),
            SharedContainer::Referenced(referenced) => {
                referenced.value_container()
            }
        }
    }

    /// Gets a [Ref] to the currently assigned allowed [Type] of the shared container (not resolved recursively)
    pub fn allowed_type(&self) -> Ref<'_, Type> {
        match self {
            SharedContainer::Owned(owned) => owned.allowed_type(),
            SharedContainer::Referenced(referenced) => {
                referenced.allowed_type()
            }
        }
    }

    /// Gets the current actual [Type] of the collapsed inner [Value]
    pub fn actual_type(&self, memory: &Memory) -> Type {
        self.with_collapsed_value(|value| value.actual_type(memory))
    }

    /// Gets a [RefMut] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    pub fn value_container_mut(&self) -> RefMut<'_, ValueContainer> {
        match self {
            SharedContainer::Owned(owned) => owned.value_container_mut(),
            SharedContainer::Referenced(referenced) => {
                referenced.value_container_mut()
            }
        }
    }

    /// Get the [SharedContainerMutability] of the inner container.
    pub fn container_mutability(&self) -> SharedContainerMutability {
        match self {
            SharedContainer::Owned(owned) => owned.container_mutability(),
            SharedContainer::Referenced(referenced) => {
                referenced.container_mutability()
            }
        }
    }

    /// Calls the provided callback with a mut reference to the recursively collapsed inner value of the shared container
    pub fn with_collapsed_value_mut<R>(
        &self,
        f: impl FnOnce(&mut Value) -> R,
    ) -> R {
        self.base_shared_container_mut().with_collapsed_value_mut(f)
    }

    /// Calls the provided callback with a reference to the recursively collapsed inner value of the shared container
    pub fn with_collapsed_value<R>(&self, f: impl FnOnce(&Value) -> R) -> R {
        self.base_shared_container().with_collapsed_value(f)
    }

    pub fn try_get_property<'a>(
        &self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, AccessError> {
        self.base_shared_container().try_get_property(key)
    }

    pub fn pointer_address(&self) -> PointerAddress {
        match self {
            SharedContainer::Owned(owned) => {
                PointerAddress::SelfOwned(owned.pointer_address().clone())
            }
            SharedContainer::Referenced(referenced) => {
                referenced.pointer_address()
            }
        }
    }

    /// Creates a new immutable [ReferencedSharedContainer] pointing to the same inner value as self.
    pub fn derive_immutable_reference(&self) -> ReferencedSharedContainer {
        match self {
            SharedContainer::Owned(owned) => owned.derive_immutable_reference(),
            SharedContainer::Referenced(referenced) => {
                referenced.derive_immutable_reference()
            }
        }
    }

    /// Tries to create a new mutable [ReferencedSharedContainer] pointing to the same inner value as this [OwnedSharedContainer].
    /// Returns an [Err] if the current reference_mutability is [ReferenceMutability::Immutable] or the container itself is not mutable
    pub fn try_derive_mutable_reference(
        &self,
    ) -> Result<ReferencedSharedContainer, UnexpectedImmutableReferenceError>
    {
        match self {
            SharedContainer::Owned(owned) => owned
                .try_derive_mutable_reference()
                .map_err(|_| UnexpectedImmutableReferenceError),
            SharedContainer::Referenced(referenced) => {
                referenced.try_derive_mutable_reference()
            }
        }
    }

    /// Returns the owned shared container if it is owned, otherwise returns an error.
    pub fn try_get_owned(
        &self,
    ) -> Result<&OwnedSharedContainer, UnexpectedSharedContainerOwnershipError>
    {
        match self {
            SharedContainer::Owned(owned) => Ok(owned),
            SharedContainer::Referenced(reference) => {
                Err(UnexpectedSharedContainerOwnershipError {
                    actual: SharedContainerOwnership::Referenced(
                        reference.reference_mutability(),
                    ),
                    expected: SharedContainerOwnership::Owned,
                })
            }
        }
    }

    /// Clones the shared container as a mutable reference if possible, otherwise as an immutable reference
    pub fn derive_with_max_mutability(&self) -> ReferencedSharedContainer {
        self.try_derive_mutable_reference()
            .unwrap_or_else(|_| self.derive_immutable_reference())
    }

    /// Checks if the shared container can be mutated by the local endpoint
    pub fn can_mutate(&self) -> bool {
        match self {
            SharedContainer::Owned(owned) => owned.can_mutate(),
            SharedContainer::Referenced(referenced) => referenced.can_mutate(),
        }
    }

    /// Returns the [SharedContainerOwnership] of this shared container
    pub fn ownership(&self) -> SharedContainerOwnership {
        match self {
            SharedContainer::Owned(_owned) => SharedContainerOwnership::Owned,
            SharedContainer::Referenced(referenced) => {
                SharedContainerOwnership::Referenced(
                    referenced.reference_mutability(),
                )
            }
        }
    }
}

/// Custom clone implementation for [SharedContainer].
/// A [SharedContainer::Owned] cannot be cloned as is, only a new reference can be created
/// A [SharedContainer::Referenced] can be cloned normally
impl Clone for SharedContainer {
    fn clone(&self) -> Self {
        match self {
            // An owned container cannot be cloned, only a new reference can be created
            SharedContainer::Owned(owned) => {
                SharedContainer::Referenced(owned.derive_with_max_mutability())
            }
            // A referenced container can be cloned
            SharedContainer::Referenced(referenced) => {
                SharedContainer::Referenced(referenced.clone())
            }
        }
    }
}

impl Display for SharedContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SharedContainer::Owned(owned) => write!(f, "{}", owned),
            SharedContainer::Referenced(referenced) => {
                write!(f, "{}", referenced)
            }
        }
    }
}

/// Two references are identical if they point to the same inner value (Rc pointer equality)
impl Identity for SharedContainer {
    fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(self.get_rc_internal(), other.get_rc_internal())
    }
}

impl Eq for SharedContainer {}

/// PartialEq corresponds to pointer equality / identity for `Reference`.
impl PartialEq for SharedContainer {
    fn eq(&self, other: &Self) -> bool {
        self.identical(other)
    }
}

impl StructuralEq for SharedContainer {
    fn structural_eq(&self, other: &Self) -> bool {
        self.inner()
            .base_shared_container()
            .value_container
            .structural_eq(
                &other.inner().base_shared_container().value_container,
            )
    }
}

impl ValueEq for SharedContainer {
    fn value_eq(&self, other: &Self) -> bool {
        self.inner()
            .base_shared_container()
            .value_container
            .value_eq(&other.inner().base_shared_container().value_container)
    }
}

impl Hash for SharedContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(self.get_rc_internal());
        ptr.hash(state); // hash the address
    }
}

impl From<OwnedSharedContainer> for SharedContainer {
    fn from(value: OwnedSharedContainer) -> Self {
        SharedContainer::Owned(value)
    }
}

impl From<ReferencedSharedContainer> for SharedContainer {
    fn from(value: ReferencedSharedContainer) -> Self {
        SharedContainer::Referenced(value)
    }
}

impl _ExposeRcInternal for SharedContainer {
    type Shared = SharedContainerInner;
    fn get_rc_internal(&self) -> &Rc<RefCell<Self::Shared>> {
        match self {
            SharedContainer::Owned(owned) => owned.get_rc_internal(),
            SharedContainer::Referenced(referenced) => {
                referenced.get_rc_internal()
            }
        }
    }
}
