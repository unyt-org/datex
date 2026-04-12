mod owned_shared_container;
mod referenced_shared_container;
mod shared_container_inner;
mod ownership;
mod endpoint_owned_shared_container;
mod external_shared_container;
pub mod shared_type_container;
pub mod shared_value_container;
mod shared_container_mutability;

use core::fmt::{Display, Formatter};
use std::ops::Deref;
pub use owned_shared_container::*;
pub use referenced_shared_container::*;
pub use shared_container_inner::*;
pub use ownership::*;
pub use endpoint_owned_shared_container::*;
pub use external_shared_container::*;
pub use shared_container_mutability::*;
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