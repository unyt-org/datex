use crate::{
    runtime::memory::Memory,
    shared_values::{
        ExternalSharedContainer, ReferenceMutability, SharedContainerInner,
        SharedContainerMutability,
        base_shared_value_container::BaseSharedValueContainer,
        errors::{SharedValueCreationError, UnexpectedImmutableReferenceError},
        internal_traits::_ExposeRcInternal,
        pointer_address::{ExternalPointerAddress, PointerAddress},
    },
    types::r#type::Type,
    values::{value::Value, value_container::ValueContainer},
};
use alloc::rc::Rc;
use core::{
    cell::{Ref, RefCell, RefMut},
    fmt::Display,
};

/// Wrapper struct for a reference to a shared value (i.e. `'shared X` or `'mut shared X`).
///
/// The inner value can either be a [SharedContainerInner::EndpointOwned] or [SharedContainerInner::External]
#[derive(Debug, Clone)]
pub struct ReferencedSharedContainer {
    /// The inner container contains the actual value which can be shared between multiple owners.
    /// This can either be a [SharedContainerInner::EndpointOwned] or a [SharedContainerInner::External]
    inner: Rc<RefCell<SharedContainerInner>>,
    /// The mutability of the reference (either `'mut shared X` or `'shared X`)
    reference_mutability: ReferenceMutability,
}

impl ReferencedSharedContainer {
    /// Creates a new mutable [ReferencedSharedContainer] from an existing mutable [Rc<RefCell<SharedContainerInner>>]
    ///
    /// IMPORTANT: this method should only be called after validating that
    /// the [SharedContainerMutability] of the inner container is mutable.
    pub(crate) fn new_mutable_unchecked(
        inner: Rc<RefCell<SharedContainerInner>>,
    ) -> Self {
        ReferencedSharedContainer {
            inner,
            reference_mutability: ReferenceMutability::Mutable,
        }
    }

    /// Creates a new immutable [ReferencedSharedContainer] from an existing mutable or immutable [Rc<RefCell<SharedContainerInner>>]
    pub(crate) fn new_immutable(
        inner: Rc<RefCell<SharedContainerInner>>,
    ) -> Self {
        ReferencedSharedContainer {
            inner,
            reference_mutability: ReferenceMutability::Immutable,
        }
    }

    /// Tries to create a new immutable [ReferencedSharedContainer] containing a [SharedContainerInner::External]
    /// Returns an [Err] if the provided [ReferenceMutability] is [ReferenceMutability::Mutable] while
    /// the [SharedContainerMutability] of the container is [SharedContainerMutability::Immutable]
    ///
    /// The caller must ensure that the [ExternalPointerAddress] does not yet exist in the [Memory]
    pub(crate) unsafe fn try_new_external_from_base_container(
        container: BaseSharedValueContainer,
        address: ExternalPointerAddress,
        reference_mutability: ReferenceMutability,
        memory: &Memory,
    ) -> Result<Self, ()> {
        // invalid reference mutability
        if reference_mutability == ReferenceMutability::Mutable
            && container.mutability == SharedContainerMutability::Immutable
        {
            return Err(());
        }

        Ok(ReferencedSharedContainer {
            inner: Rc::new(RefCell::new(SharedContainerInner::External(
                unsafe {
                    ExternalSharedContainer::create_external_shared_container(
                        container, address, memory,
                    )
                },
            ))),
            reference_mutability,
        })
    }

    /// Creates a new immutable [ReferencedSharedContainer] containing a [SharedContainerInner::External]
    /// with the provided [ValueContainer] and [ExternalPointerAddress].
    /// Automatically infers the allowed type of the shared container from the provided [ValueContainer].
    ///
    /// The caller must ensure that the [ExternalPointerAddress] does not yet exist in the [Memory]
    pub(crate) unsafe fn new_immutable_external_with_inferred_allowed_type(
        value_container: ValueContainer,
        address: ExternalPointerAddress,
        memory: &Memory,
    ) -> Self {
        unsafe {
            ReferencedSharedContainer::try_new_external_from_base_container(
                BaseSharedValueContainer::new_with_inferred_allowed_type(
                    value_container,
                    SharedContainerMutability::Immutable,
                    memory,
                ),
                address,
                ReferenceMutability::Immutable,
                memory,
            )
            .unwrap()
        }
    }

    /// Tries to create a new immutable [ReferencedSharedContainer] containing a [SharedContainerInner::External]
    /// with the provided [ValueContainer] and [ExternalPointerAddress].
    ///
    /// Sets the provided [SharedContainerMutability] and allowed [Type].
    /// If the allowed [TypeDefinition] is not a superset of the [ValueContainer]'s allowed type,
    /// an error is returned
    ///
    /// The caller must ensure that the [ExternalPointerAddress] does not yet exist in the [Memory]
    pub(crate) unsafe fn try_new_external(
        value_container: ValueContainer,
        address: ExternalPointerAddress,
        mutability: SharedContainerMutability,
        allowed_type: Type,
        memory: &Memory,
    ) -> Result<Self, SharedValueCreationError> {
        Ok(unsafe {
            ReferencedSharedContainer::try_new_external_from_base_container(
                BaseSharedValueContainer::try_new(
                    value_container,
                    allowed_type,
                    mutability,
                )?,
                address,
                ReferenceMutability::Immutable,
                memory,
            )
            .unwrap()
        })
    }

    pub fn inner(&self) -> Ref<'_, SharedContainerInner> {
        self.inner.borrow()
    }
    pub fn inner_mut(&self) -> RefMut<'_, SharedContainerInner> {
        self.inner.borrow_mut()
    }

    /// Gets a [Ref] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    pub fn base_shared_container(&self) -> Ref<'_, BaseSharedValueContainer> {
        Ref::map(self.inner(), |inner| inner.base_shared_container())
    }

    /// Gets a [RefMut] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    pub fn base_shared_container_mut(
        &self,
    ) -> RefMut<'_, BaseSharedValueContainer> {
        RefMut::map(self.inner_mut(), |inner| inner.base_shared_container_mut())
    }

    /// Gets a [Ref] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    pub fn value_container(&self) -> Ref<'_, ValueContainer> {
        Ref::map(self.base_shared_container(), |base_shared_container| {
            &base_shared_container.value_container
        })
    }

    /// Gets a [RefMut] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    pub fn value_container_mut(&self) -> RefMut<'_, ValueContainer> {
        RefMut::map(self.base_shared_container_mut(), |base_shared_container| {
            &mut base_shared_container.value_container
        })
    }

    /// Calls the provided callback with a mut reference to the recursively collapsed inner value of the shared container
    pub fn with_collapsed_value_mut<R>(
        &self,
        f: impl FnOnce(&mut Value) -> R,
    ) -> R {
        self.inner_mut()
            .base_shared_container_mut()
            .with_collapsed_value_mut(f)
    }

    /// Calls the provided callback with a reference to the recursively collapsed inner value of the shared container
    pub fn with_collapsed_value<R>(&self, f: impl FnOnce(&Value) -> R) -> R {
        self.inner().base_shared_container().with_collapsed_value(f)
    }

    /// Gets a [Ref] to the currently assigned allowed [TypeDefinition] of the shared container (not resolved recursively)
    pub fn allowed_type(&self) -> Ref<'_, Type> {
        Ref::map(self.base_shared_container(), |base_shared_container| {
            &base_shared_container.allowed_type
        })
    }

    /// Get the inner [PointerAddress].
    pub fn pointer_address(&self) -> PointerAddress {
        self.inner().pointer_address()
    }

    /// Get the [SharedContainerMutability] of the inner [SelfOwnedSharedContainer].
    pub fn container_mutability(&self) -> SharedContainerMutability {
        self.inner().base_shared_container().mutability.clone()
    }

    /// Creates a new immutable [ReferencedSharedContainer] pointing to the same inner value as self.
    pub fn derive_immutable_reference(&self) -> ReferencedSharedContainer {
        ReferencedSharedContainer {
            inner: self.inner.clone(),
            reference_mutability: ReferenceMutability::Immutable,
        }
    }

    /// Tries to create a new mutable [ReferencedSharedContainer] pointing to the same inner value as self.
    /// Returns an [Err] if the current reference_mutability is [ReferenceMutability::Immutable]
    pub fn try_derive_mutable_reference(
        &self,
    ) -> Result<ReferencedSharedContainer, UnexpectedImmutableReferenceError>
    {
        match self.reference_mutability {
            ReferenceMutability::Immutable => {
                Err(UnexpectedImmutableReferenceError)
            }
            ReferenceMutability::Mutable => Ok(self.clone()),
        }
    }

    /// Checks if the reference can be mutated by the local endpoint
    pub(crate) fn can_mutate(&self) -> bool {
        self.reference_mutability == ReferenceMutability::Mutable
    }

    /// Returns the [ReferenceMutability] of this reference
    pub fn reference_mutability(&self) -> ReferenceMutability {
        self.reference_mutability
    }
}

impl Display for ReferencedSharedContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}{}",
            match self.reference_mutability {
                ReferenceMutability::Immutable => "'",
                ReferenceMutability::Mutable => "'mut ",
            },
            self.inner().base_shared_container(),
        )
    }
}

impl _ExposeRcInternal for ReferencedSharedContainer {
    type Shared = SharedContainerInner;
    fn get_rc_internal(&self) -> &Rc<RefCell<Self::Shared>> {
        &self.inner
    }
}
