pub mod datex_proxy;
mod common;

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::{
        ExternalSharedContainer, PointerAddress, ReferenceMutability,
        RemotePointerAddress, SharedContainerInner, SharedContainerMutability,
        base_shared_value_container::BaseSharedValueContainer,
        errors::{SharedValueCreationError, UnexpectedImmutableReferenceError},
        internal_traits::_ExposeRcInternal,
    },
    traits::{
        identity::Identity, structural_eq::StructuralEq, value_eq::ValueEq,
    },
    types::type_definition::TypeDefinition,
    values::{value::Value, value_container::ValueContainer},
};
use alloc::rc::Rc;
use core::{
    cell::{Ref, RefCell, RefMut},
    fmt::Display,
    hash::{Hash, Hasher},
};
use crate::shared_values::shared_container_common::SharedContainerCommon;
use crate::prelude::*;

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
    container_mutability: SharedContainerMutability,
    /// Field used internally to indicate that this reference should be treated as a move in the context of the compiler
    move_indicator: bool,
}

impl ReferencedSharedContainer {
    /// Creates a new mutable [ReferencedSharedContainer] from an existing mutable [Rc<RefCell<SharedContainerInner>>]
    ///
    /// IMPORTANT: this method should only be called after validating that
    /// the [SharedContainerMutability] of the inner container is mutable.
    pub(crate) fn new_mutable_unchecked(
        inner: Rc<RefCell<SharedContainerInner>>,
    ) -> Self {
        let container_mutability = inner.borrow().base_shared_container().mutability().clone();

        ReferencedSharedContainer {
            inner,
            reference_mutability: ReferenceMutability::Mutable,
            container_mutability,
            move_indicator: false,
        }
    }

    /// Creates a new immutable [ReferencedSharedContainer] from an existing mutable or immutable [Rc<RefCell<SharedContainerInner>>]
    pub(crate) fn new_immutable(
        inner: Rc<RefCell<SharedContainerInner>>,
    ) -> Self {
        let container_mutability = inner.borrow().base_shared_container().mutability().clone();

        ReferencedSharedContainer {
            inner,
            reference_mutability: ReferenceMutability::Immutable,
            container_mutability,
            move_indicator: false,
        }
    }

    /// Tries to create a new immutable [ReferencedSharedContainer] containing a [SharedContainerInner::External]
    /// Returns an [Err] if the provided [ReferenceMutability] is [ReferenceMutability::Mutable] while
    /// the [SharedContainerMutability] of the container is [SharedContainerMutability::Immutable]
    /// # Safety
    /// The caller must ensure that the [RemotePointerAddress] does not yet exist in the [SharedReferencesCache]
    pub(crate) unsafe fn try_new_remote_from_base_container(
        container: BaseSharedValueContainer,
        address: RemotePointerAddress,
        reference_mutability: ReferenceMutability,
    ) -> Result<Self, ()> {
        let container_mutability = container.mutability().clone();
        // invalid reference mutability
        if reference_mutability == ReferenceMutability::Mutable
            && *container.mutability() == SharedContainerMutability::Immutable
        {
            return Err(());
        }

        Ok(ReferencedSharedContainer {
            inner: Rc::new(RefCell::new(SharedContainerInner::External(
                unsafe {
                    ExternalSharedContainer::new(
                        container, address,
                    )
                },
            ))),
            reference_mutability,
            container_mutability,
            move_indicator: false,
        })
    }

    /// Creates a new immutable [ReferencedSharedContainer] containing a [SharedContainerInner::External]
    /// with the provided [ValueContainer] and [RemotePointerAddress].
    /// Automatically infers the allowed type of the shared container from the provided [ValueContainer].
    /// # Safety
    /// The caller must ensure that the [RemotePointerAddress] does not yet exist in the [SharedReferencesCache]
    pub(crate) unsafe fn new_immutable_external_with_inferred_allowed_type(
        value_container: ValueContainer,
        address: RemotePointerAddress,
    ) -> Self {
        unsafe {
            ReferencedSharedContainer::try_new_remote_from_base_container(
                BaseSharedValueContainer::new_with_inferred_allowed_type(
                    value_container,
                    SharedContainerMutability::Immutable,
                ),
                address,
                ReferenceMutability::Immutable,
            )
            .unwrap()
        }
    }

    /// Tries to create a new immutable [ReferencedSharedContainer] containing a [SharedContainerInner::External]
    /// with the provided [ValueContainer] and [RemotePointerAddress].
    ///
    /// Sets the provided [SharedContainerMutability] and allowed [TypeDefinition].
    /// If the allowed [TypeDefinition] is not a superset of the [ValueContainer]'s allowed type,
    /// an error is returned
    /// # Safety
    /// The caller must ensure that the [RemotePointerAddress] does not yet exist in the [SharedReferencesCache]
    pub(crate) unsafe fn try_new_external(
        value_container: ValueContainer,
        address: RemotePointerAddress,
        mutability: SharedContainerMutability,
        allowed_type: TypeDefinition,
    ) -> Result<Self, SharedValueCreationError> {
        Ok(unsafe {
            ReferencedSharedContainer::try_new_remote_from_base_container(
                BaseSharedValueContainer::try_new(
                    value_container,
                    allowed_type,
                    mutability,
                )?,
                address,
                ReferenceMutability::Immutable,
            )
            .unwrap()
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

    /// Get the inner [PointerAddress].
    pub fn pointer_address(&self) -> PointerAddress {
        self.inner().pointer_address()
    }


    /// Creates a new immutable [ReferencedSharedContainer] pointing to the same inner value as self.
    pub fn derive_immutable_reference(&self) -> ReferencedSharedContainer {
        ReferencedSharedContainer {
            inner: self.inner.clone(),
            reference_mutability: ReferenceMutability::Immutable,
            container_mutability: self.container_mutability(),
            move_indicator: false,
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

    /// Returns the [ReferenceMutability] of this reference
    pub fn reference_mutability(&self) -> ReferenceMutability {
        self.reference_mutability
    }


    /// Sets the move indicator flag that signals that this should be treated as a move in the context
    /// of the compiler
    /// Note: this should only be set on containers with an owned address
    pub(super) unsafe fn set_move_indicator(&mut self) {
        self.move_indicator = true;
    }

    /// Returns true if the move indicator is set
    pub fn treat_as_move(&self) -> bool {
        self.move_indicator
    }

    pub unsafe fn change_address(&self, new_address: PointerAddress) {
        assert_eq!(self.move_indicator, false);
        unsafe {
            self.inner_mut().change_address(new_address)
        }
    }

    pub fn to_string_omit_content(&self) -> String {
        format!(
            "{}(...)",
            match self.reference_mutability {
                ReferenceMutability::Immutable => "'",
                ReferenceMutability::Mutable => "'mut ",
            },
        )
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

impl Identity for ReferencedSharedContainer {
    fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for ReferencedSharedContainer {}

/// PartialEq corresponds to pointer equality / identity for `Reference`.
impl PartialEq for ReferencedSharedContainer {
    fn eq(&self, other: &Self) -> bool {
        self.identical(other)
    }
}

impl Hash for ReferencedSharedContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(self.get_rc_internal());
        ptr.hash(state); // hash the address
    }
}

impl StructuralEq for ReferencedSharedContainer {
    fn structural_eq(&self, other: &Self) -> bool {
        self.inner()
            .base_shared_container()
            .value_container()
            .structural_eq(
                other.inner().base_shared_container().value_container(),
            )
    }
}

impl ValueEq for ReferencedSharedContainer {
    fn value_eq(&self, other: &Self) -> bool {
        self.inner()
            .base_shared_container()
            .value_container()
            .value_eq(other.inner().base_shared_container().value_container())
    }
}
