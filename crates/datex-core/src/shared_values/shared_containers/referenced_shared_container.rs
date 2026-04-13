use alloc::rc::Rc;
use core::cell::RefCell;
use core::cell::Ref;
use core::fmt::Display;
use std::cell::RefMut;
use crate::shared_values::pointer_address::PointerAddress;
use crate::shared_values::shared_container::{SelfOwnedSharedContainer, SharedContainerInner, SharedContainerMutability};
use crate::shared_values::shared_containers::expose_rc_internal::ExposeRcInternal;
use crate::shared_values::shared_containers::ReferenceMutability;

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
    pub(crate) fn new_mutable_unchecked(inner: Rc<RefCell<SharedContainerInner>>) -> Self {
        ReferencedSharedContainer {
            inner,
            reference_mutability: ReferenceMutability::Mutable,
        }
    }

    /// Creates a new immutable [ReferencedSharedContainer] from an existing mutable or immmutable [Rc<RefCell<SharedContainerInner>>]
    pub(crate) fn new_immutable(inner: Rc<RefCell<SharedContainerInner>>) -> Self {
        ReferencedSharedContainer {
            inner,
            reference_mutability: ReferenceMutability::Immutable,
        }
    }

    pub fn as_inner(&self) -> Ref<SharedContainerInner> {
        self.inner.borrow()
    }
    pub fn as_inner_mut(&self) -> RefMut<SharedContainerInner> {
        self.inner.borrow_mut()
    }

    /// Get the inner [PointerAddress].
    pub fn pointer_address(&self) -> PointerAddress {
        self.as_inner().pointer_address()
    }

    /// Get the [SharedContainerMutability] of the inner [SelfOwnedSharedContainer].
    pub fn container_mutability(&self) -> SharedContainerMutability {
        self.as_inner().value().mutability.clone()
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
    pub fn try_derive_mutable_reference(&self) -> Result<ReferencedSharedContainer, ()> {
        match self.reference_mutability {
            ReferenceMutability::Immutable => Err(()),
            ReferenceMutability::Mutable => Ok(self.clone()),
        }
    }

    /// Checks if the reference can be mutated by the local endpoint
    pub(crate) fn can_mutate(&self) -> bool {
        self.reference_mutability == ReferenceMutability::Mutable
    }

    /// Returns the [ReferenceMutability] of this reference
    pub fn reference_mutability(&self) -> ReferenceMutability {
        self.reference_mutability.clone()
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
            self.as_inner().value(),
        )
    }
}

impl ExposeRcInternal for ReferencedSharedContainer {
    type Shared = SharedContainerInner;
    fn get_rc_internal(&self) -> &Rc<RefCell<Self::Shared>> {
        &self.inner
    }
}