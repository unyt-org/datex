mod owned_shared_container;
mod referenced_shared_container;
mod shared_container_inner;
mod ownership;
mod self_owned_shared_container;
mod external_shared_container;
pub mod shared_type_container;
pub mod shared_value_container;
mod shared_container_mutability;
mod expose_rc_internal;
mod shared_container_containing_type;
// IMPORTANT: don't expose this module, for internal use only

use alloc::rc::Rc;
use core::fmt::{Display, Formatter};
use core::cell::{Ref, RefCell};
use core::ops::Deref;
use std::hash::{Hash, Hasher};
pub use owned_shared_container::*;
pub use referenced_shared_container::*;
pub use shared_container_inner::*;
pub use ownership::*;
pub use self_owned_shared_container::*;
pub use external_shared_container::*;
pub use shared_container_mutability::*;
pub use shared_container_containing_type::*;
use crate::shared_values::pointer_address::{SelfOwnedPointerAddress, PointerAddress};
use crate::shared_values::shared_containers::expose_rc_internal::ExposeRcInternal;
use crate::traits::identity::Identity;
use crate::traits::structural_eq::StructuralEq;
use crate::traits::value_eq::ValueEq;
use crate::values::core_value::CoreValue;
use crate::values::value_container::ValueContainer;


impl Deref for SharedContainerContainingType {
    type Target = SharedContainer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


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
    /// a [SharedContainerMutability], and an [SelfOwnedPointerAddress].
    ///
    /// The allowed type is inferred from the value_container's allowed type.
    pub fn new_owned_with_inferred_allowed_type(
        value_container: ValueContainer,
        mutability: SharedContainerMutability,
        address: SelfOwnedPointerAddress,
    ) -> Self {
        SharedContainer::Owned(OwnedSharedContainer::new_with_inferred_allowed_type(
            value_container,
            mutability,
            address,
        ))
    }


    pub fn as_inner(&self) -> Ref<SharedContainerInner> {
        match self {
            SharedContainer::Owned(owned) => owned.as_inner(),
            SharedContainer::Referenced(referenced) => referenced.as_inner(),
        }
    }

    pub fn pointer_address(&self) -> PointerAddress {
        match self {
            SharedContainer::Owned(owned) => PointerAddress::EndpointOwned(owned.pointer_address().clone()),
            SharedContainer::Referenced(referenced) => referenced.pointer_address(),
        }
    }

    /// Creates a new immutable [ReferencedSharedContainer] pointing to the same inner value as self.
    pub fn derive_immutable_reference(&self) -> ReferencedSharedContainer {
        match self {
            SharedContainer::Owned(owned) => owned.derive_immutable_reference(),
            SharedContainer::Referenced(referenced) => referenced.derive_immutable_reference(),
        }
    }

    /// Tries to create a new mutable [ReferencedSharedContainer] pointing to the same inner value as this [OwnedSharedContainer].
    /// Returns an [Err] if the current reference_mutability is [ReferenceMutability::Immutable] or the container itself is not mutable
    pub fn try_derive_mutable_reference(&self) -> Result<ReferencedSharedContainer, ()> {
        match self {
            SharedContainer::Owned(owned) => owned.try_derive_mutable_reference(),
            SharedContainer::Referenced(referenced) => referenced.try_derive_mutable_reference(),
        }
    }

    /// Returns the owned shared container if it is owned, otherwise returns an error.
    pub fn try_get_owned(&self) -> Result<&OwnedSharedContainer, ()> {
        match self {
            SharedContainer::Owned(owned) => Ok(owned),
            SharedContainer::Referenced(_) => Err(()),
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
            SharedContainer::Owned(owned) => SharedContainerOwnership::Owned,
            SharedContainer::Referenced(referenced) => SharedContainerOwnership::Referenced(referenced.reference_mutability())
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
            SharedContainer::Owned(owned) => SharedContainer::Referenced(owned.derive_with_max_mutability()),
            // A referenced container can be cloned
            SharedContainer::Referenced(referenced) => SharedContainer::Referenced(referenced.clone()),
        }
    }
}

impl Display for SharedContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SharedContainer::Owned(owned) => write!(f, "{}", owned),
            SharedContainer::Referenced(referenced) => write!(f, "{}", referenced),
        }
    }
}


/// Two references are identical if they point to the same inner value (Rc pointer equality)
impl Identity for SharedContainer {
    fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.get_rc_internal(), &other.get_rc_internal())
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
        self.as_inner().value().value_container.structural_eq(&other.as_inner().value().value_container)
    }
}


impl ValueEq for SharedContainer {
    fn value_eq(&self, other: &Self) -> bool {
        self.as_inner().value().value_container.value_eq(&other.as_inner().value().value_container)
    }
}

impl Hash for SharedContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(&self.get_rc_internal());
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

impl ExposeRcInternal for SharedContainer {
    type Shared = SharedContainerInner;
    fn get_rc_internal(&self) -> &Rc<RefCell<Self::Shared>> {
        match self {
            SharedContainer::Owned(owned) => owned.get_rc_internal(),
            SharedContainer::Referenced(referenced) => referenced.get_rc_internal(),
        }
    }
}