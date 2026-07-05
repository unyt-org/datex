use crate::{
    shared_values::{
        SharedContainer, SharedContainerInner, SharedContainerMutability,
        SharedContainerOwnership,
        base_shared_value_container::BaseSharedValueContainer,
        shared_container_common::SharedContainerCommon,
    },
    types::type_definition::TypeDefinition,
    values::value_container::ValueContainer,
};
use core::cell::{Ref, RefMut};

impl SharedContainerCommon for SharedContainer {
    /// Get the [SharedContainerMutability] of the inner container.
    fn container_mutability(&self) -> SharedContainerMutability {
        match self {
            SharedContainer::Owned(owned) => owned.container_mutability(),
            SharedContainer::Referenced(referenced) => {
                referenced.container_mutability()
            }
        }
    }

    /// Gets a [Ref] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    fn value_container(&self) -> Ref<'_, ValueContainer> {
        match self {
            SharedContainer::Owned(owned) => owned.value_container(),
            SharedContainer::Referenced(referenced) => {
                referenced.value_container()
            }
        }
    }

    /// Gets a [RefMut] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    fn value_container_mut(&self) -> RefMut<'_, ValueContainer> {
        match self {
            SharedContainer::Owned(owned) => owned.value_container_mut(),
            SharedContainer::Referenced(referenced) => {
                referenced.value_container_mut()
            }
        }
    }

    /// Gets a [Ref] to the currently assigned allowed [TypeDefinition] of the shared container (not resolved recursively)
    fn allowed_type(&self) -> Ref<'_, TypeDefinition> {
        match self {
            SharedContainer::Owned(owned) => owned.allowed_type(),
            SharedContainer::Referenced(referenced) => {
                referenced.allowed_type()
            }
        }
    }

    fn is_borrowed(&self) -> bool {
        match self {
            SharedContainer::Owned(owned) => owned.is_borrowed(),
            SharedContainer::Referenced(referenced) => referenced.is_borrowed(),
        }
    }

    /// Checks if the shared container can be mutated by the local endpoint
    fn can_mutate(&self) -> bool {
        match self {
            SharedContainer::Owned(owned) => owned.can_mutate(),
            SharedContainer::Referenced(referenced) => referenced.can_mutate(),
        }
    }

    fn inner(&self) -> Ref<'_, SharedContainerInner> {
        match self {
            SharedContainer::Owned(owned) => owned.inner(),
            SharedContainer::Referenced(referenced) => referenced.inner(),
        }
    }

    fn inner_mut(&self) -> RefMut<'_, SharedContainerInner> {
        match self {
            SharedContainer::Owned(owned) => owned.inner_mut(),
            SharedContainer::Referenced(referenced) => referenced.inner_mut(),
        }
    }

    /// Gets a [Ref] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    fn base_shared_container(&self) -> Ref<'_, BaseSharedValueContainer> {
        match self {
            SharedContainer::Owned(owned) => owned.base_shared_container(),
            SharedContainer::Referenced(referenced) => {
                referenced.base_shared_container()
            }
        }
    }
    /// Gets a [RefMut] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    fn base_shared_container_mut(
        &self,
    ) -> RefMut<'_, BaseSharedValueContainer> {
        match self {
            SharedContainer::Owned(owned) => owned.base_shared_container_mut(),
            SharedContainer::Referenced(referenced) => {
                referenced.base_shared_container_mut()
            }
        }
    }

    /// Returns the [SharedContainerOwnership] of this shared container
    fn ownership(&self) -> SharedContainerOwnership {
        match self {
            SharedContainer::Owned(owned) => owned.ownership(),
            SharedContainer::Referenced(referenced) => referenced.ownership(),
        }
    }
}
