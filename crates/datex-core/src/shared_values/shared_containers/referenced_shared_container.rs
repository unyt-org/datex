use alloc::rc::Rc;
use core::cell::RefCell;
use core::cell::Ref;
use core::fmt::Display;
use crate::shared_values::pointer_address::PointerAddress;
use crate::shared_values::shared_container::{OwnedSharedContainer, SharedContainerInner, SharedContainerMutability};
use crate::shared_values::shared_containers::ReferenceMutability;

/// Wrapper struct for a reference to a shared value (i.e. `'shared X` or `'mut shared X`).
///
/// The inner value can either be a [SharedContainerInner::EndpointOwned] or [SharedContainerInner::External]
#[derive(Debug, Clone)]
pub struct ReferencedSharedContainer {
    /// The inner container contains the actual value which can be shared between multiple owners.
    /// This can either be a [SharedContainerInner::EndpointOwned] or a [SharedContainerInner::External]
    pub(crate) inner: Rc<RefCell<SharedContainerInner>>,
    /// The mutability of the reference (either `'mut shared X` or `'shared X`)
    pub(crate) reference_mutability: ReferenceMutability,
}

impl ReferencedSharedContainer {
    
    pub fn as_inner(&self) -> Ref<SharedContainerInner> {
        self.inner.borrow()
    }

    /// Get the inner [PointerAddress].
    pub fn pointer_address(&self) -> PointerAddress {
        self.as_inner().pointer_address()
    }
    
    /// Get the [SharedContainerMutability] of the inner [EndpointOwnedSharedContainer].
    pub fn container_mutability(&self) -> SharedContainerMutability {
        self.as_inner().value().mutability.clone()
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