use std::cell::{Ref, RefMut};
use crate::shared_values::{SharedContainerInner, SharedContainerMutability, SharedContainerOwnership};
use crate::shared_values::base_shared_value_container::BaseSharedValueContainer;
use crate::types::type_definition::TypeDefinition;
use crate::values::value_container::ValueContainer;

pub trait SharedContainerCommon {
    /// Get the [SharedContainerMutability] of the container
    fn container_mutability(&self) -> SharedContainerMutability;

    /// Gets a [Ref] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    fn value_container(&self) -> Ref<'_, ValueContainer>;

    /// Gets a [RefMut] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    fn value_container_mut(&self) -> RefMut<'_, ValueContainer>;

    /// Gets a [Ref] to the currently assigned allowed [TypeDefinition] of the shared container (not resolved recursively)
    fn allowed_type(&self) -> Ref<'_, TypeDefinition>;

    /// Checks if the Rc in the shared container is currently borrowed mutably or immutably
    fn is_borrowed(&self) -> bool;

    /// Checks if the reference can be mutated by the local endpoint
    fn can_mutate(&self) -> bool;

    fn inner(&self) -> Ref<'_, SharedContainerInner>;
    fn inner_mut(&self) -> RefMut<'_, SharedContainerInner>;

    /// Gets a [Ref] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    fn base_shared_container(&self) -> Ref<'_, BaseSharedValueContainer>;

    /// Gets a [RefMut] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    fn base_shared_container_mut(
        &self,
    ) -> RefMut<'_, BaseSharedValueContainer>;

    /// Returns the [SharedContainerOwnership] of this shared container
    fn ownership(&self) -> SharedContainerOwnership;
}