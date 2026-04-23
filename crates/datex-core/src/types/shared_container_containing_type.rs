use crate::{
    shared_values::shared_containers::SharedContainer, types::r#type::Type,
    values::core_value::CoreValue,
};
use core::ops::Deref;
use serde::Serialize;

/// A wrapper around an [SharedContainer] which guarantees
/// that the contained value is always a [CoreValue::Type]
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize)]
pub struct SharedContainerContainingType(SharedContainer);

impl Deref for SharedContainerContainingType {
    type Target = SharedContainer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SharedContainerContainingType {
    /// Creates a new [SharedContainerContainingType] from a [SharedContainer] without checking the constraint.
    /// The caller must ensure that the constraint for [SharedContainerContainingType] is satisfied
    /// (i.e. the allowed type of the container is a [StructuralTypeDefinition::Type])
    pub unsafe fn new_unchecked(container: SharedContainer) -> Self {
        SharedContainerContainingType(container)
    }

    /// Calls the provided callback with a reference to the recursively collapsed inner [Type] value of the shared container
    /// The [SharedContainerContainingType] guarantees that the inner value is always a [CoreValue::Type], so this method can never panic.
    pub fn with_collapsed_type_value<R>(
        &self,
        f: impl FnOnce(&Type) -> R,
    ) -> R {
        self.0.with_collapsed_value(|value| match &value.inner {
            CoreValue::Type(ty) => f(ty),
            _ => unreachable!("The constraint for SharedContainerContainingType guarantees that the inner value is always a CoreValue::Type")
        })
    }
}
