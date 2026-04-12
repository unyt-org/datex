mod owned_shared_container;
mod referenced_shared_container;
mod shared_container_inner;
mod ownership;
mod endpoint_owned_shared_container;
mod external_shared_container;
pub mod shared_type_container;
pub mod shared_value_container;
mod shared_container_mutability;

use alloc::rc::Rc;
use core::fmt::{Display, Formatter};
use core::cell::{Ref, RefCell};
use core::ops::Deref;
use std::hash::{Hash, Hasher};
pub use owned_shared_container::*;
pub use referenced_shared_container::*;
pub use shared_container_inner::*;
pub use ownership::*;
pub use endpoint_owned_shared_container::*;
pub use external_shared_container::*;
pub use shared_container_mutability::*;
use crate::shared_values::pointer::ExternalPointer;
use crate::shared_values::pointer_address::{ExternalPointerAddress, PointerAddress};
use crate::traits::identity::Identity;
use crate::traits::structural_eq::StructuralEq;
use crate::traits::value_eq::ValueEq;
use crate::values::core_value::CoreValue;

/// Top-level wrapper for any shared container, distinguishing between
/// containers that are guaranteed to contain a [CoreValue::Type] and normal value containers without this constraint.
/// Can be trivially dereferenced to an [OwnedOrReferencedSharedContainer] regardless of the variant.
#[derive(Debug, Clone)]
pub enum SharedContainer {
    /// A normal [OwnedOrReferencedSharedContainer] without any guarantees about the contained value
    Value(OwnedOrReferencedSharedContainer),
    /// An [OwnedOrReferencedSharedContainer] which is guaranteed to always have an inner value of type [CoreValue::Type]
    Type(SharedContainerContainingType),
}

impl Deref for SharedContainer {
    type Target = OwnedOrReferencedSharedContainer;
    fn deref(&self) -> &Self::Target {
        match self {
            SharedContainer::Value(shared_container) => shared_container,
            SharedContainer::Type(type_container) => type_container.deref(),
        }
    }
}

impl PartialEq for SharedContainer {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl Eq for SharedContainer {}

impl Identity for SharedContainer {
    fn identical(&self, other: &Self) -> bool {
        self.deref().identical(other.deref())
    }
}

/// A wrapper around an [OwnedOrReferencedSharedContainer] which guarantees
/// that the contained value is always a [CoreValue::Type]
#[derive(Debug, Clone)]
pub struct SharedContainerContainingType(OwnedOrReferencedSharedContainer);

impl Deref for SharedContainerContainingType {
    type Target = OwnedOrReferencedSharedContainer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


/// Top-level wrapper for any owned or referenced shared container,
/// which can either be an owned shared container or a reference to a shared container.
#[derive(Debug)]
pub enum OwnedOrReferencedSharedContainer {
    /// An owned shared container (`shared X`). This is always points to a [SharedContainerInner::EndpointOwned]
    Owned(OwnedSharedContainer),
    /// A referenced shared container (`'shared X` or `'mut shared X`).
    /// This can point to either a [SharedContainerInner::EndpointOwned] or a [SharedContainerInner::External]
    Referenced(ReferencedSharedContainer),
}

impl OwnedOrReferencedSharedContainer {
    /// Get a reference to the inner [Rc<RefCell<SharedContainerInner>>]
    pub fn inner_rc(&self) -> &Rc<RefCell<SharedContainerInner>> {
        match self {
            OwnedOrReferencedSharedContainer::Owned(owned) => owned.inner_rc(),
            OwnedOrReferencedSharedContainer::Referenced(referenced) => referenced.inner_rc(),
        }
    }

    pub fn as_inner(&self) -> Ref<SharedContainerInner> {
        match self {
            OwnedOrReferencedSharedContainer::Owned(owned) => owned.as_inner(),
            OwnedOrReferencedSharedContainer::Referenced(referenced) => referenced.as_inner(),
        }
    }

    pub fn pointer_address(&self) -> PointerAddress {
        match self {
            OwnedOrReferencedSharedContainer::Owned(owned) => PointerAddress::EndpointOwned(owned.pointer_address().clone()),
            OwnedOrReferencedSharedContainer::Referenced(referenced) => referenced.pointer_address(),
        }
    }

    /// Creates a new immutable [ReferencedSharedContainer] pointing to the same inner value as self.
    pub fn derive_immutable_reference(&self) -> ReferencedSharedContainer {
        match self {
            OwnedOrReferencedSharedContainer::Owned(owned) => owned.derive_immutable_reference(),
            OwnedOrReferencedSharedContainer::Referenced(referenced) => referenced.derive_immutable_reference(),
        }
    }

    /// Tries to create a new mutable [ReferencedSharedContainer] pointing to the same inner value as this [OwnedSharedContainer].
    /// Returns an [Err] if the current reference_mutability is [ReferenceMutability::Immutable] or the container itself is not mutable
    pub fn try_derive_mutable_reference(&self) -> Result<ReferencedSharedContainer, ()> {
        match self {
            OwnedOrReferencedSharedContainer::Owned(owned) => owned.try_derive_mutable_reference(),
            OwnedOrReferencedSharedContainer::Referenced(referenced) => referenced.try_derive_mutable_reference(),
        }
    }

    /// Returns the owned shared container if it is owned, otherwise returns an error.
    pub fn try_get_owned(&self) -> Result<&OwnedSharedContainer, ()> {
        match self {
            OwnedOrReferencedSharedContainer::Owned(owned) => Ok(owned),
            OwnedOrReferencedSharedContainer::Referenced(_) => Err(()),
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
            OwnedOrReferencedSharedContainer::Owned(owned) => owned.can_mutate(),
            OwnedOrReferencedSharedContainer::Referenced(referenced) => referenced.can_mutate(),
        }
    }
}

/// Custom clone implementation for [OwnedOrReferencedSharedContainer].
/// A [OwnedOrReferencedSharedContainer::Owned] cannot be cloned as is, only a new reference can be created
/// A [OwnedOrReferencedSharedContainer::Referenced] can be cloned normally
impl Clone for OwnedOrReferencedSharedContainer {
    fn clone(&self) -> Self {
        match self {
            // An owned container cannot be cloned, only a new reference can be created
            OwnedOrReferencedSharedContainer::Owned(owned) => OwnedOrReferencedSharedContainer::Referenced(owned.derive_with_max_mutability()),
            // A referenced container can be cloned
            OwnedOrReferencedSharedContainer::Referenced(referenced) => OwnedOrReferencedSharedContainer::Referenced(referenced.clone()),
        }
    }
}

impl Display for OwnedOrReferencedSharedContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            OwnedOrReferencedSharedContainer::Owned(owned) => write!(f, "{}", owned),
            OwnedOrReferencedSharedContainer::Referenced(referenced) => write!(f, "{}", referenced),
        }
    }
}


/// Two references are identical if they point to the same inner value (Rc pointer equality)
impl Identity for OwnedOrReferencedSharedContainer {
    fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner_rc(), &other.inner_rc())
    }
}

impl Eq for OwnedOrReferencedSharedContainer {}

/// PartialEq corresponds to pointer equality / identity for `Reference`.
impl PartialEq for OwnedOrReferencedSharedContainer {
    fn eq(&self, other: &Self) -> bool {
        self.identical(other)
    }
}

impl StructuralEq for OwnedOrReferencedSharedContainer {
    fn structural_eq(&self, other: &Self) -> bool {
        self.as_inner().value().value_container.structural_eq(&other.as_inner().value().value_container)
    }
}


impl ValueEq for OwnedOrReferencedSharedContainer {
    fn value_eq(&self, other: &Self) -> bool {
        self.as_inner().value().value_container.value_eq(&other.as_inner().value().value_container)
    }
}

impl Hash for OwnedOrReferencedSharedContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(&self.inner_rc());
        ptr.hash(state); // hash the address
    }
}

impl From<OwnedSharedContainer> for OwnedOrReferencedSharedContainer {
    fn from(value: OwnedSharedContainer) -> Self {
        OwnedOrReferencedSharedContainer::Owned(value)
    }
}

impl From<ReferencedSharedContainer> for OwnedOrReferencedSharedContainer {
    fn from(value: ReferencedSharedContainer) -> Self {
        OwnedOrReferencedSharedContainer::Referenced(value)
    }
}

impl From<OwnedOrReferencedSharedContainer> for SharedContainer {
    fn from(value: OwnedOrReferencedSharedContainer) -> Self {
        SharedContainer::Value(value)
    }
}