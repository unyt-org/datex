use core::ops::Deref;

use crate::{
    libs::core::{
        core_lib_id::CoreLibId,
        type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    },
    shared_values::{
        pointer_address::PointerAddress, shared_containers::SharedContainer,
    },
    types::{nominal_type_definition::NominalTypeDefinition, r#type::Type},
    values::core_value::CoreValue,
};

/// A wrapper around an [SharedContainer] which guarantees
/// that the contained value is always a [CoreValue::Type]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SharedContainerContainingType(SharedContainer);

impl Deref for SharedContainerContainingType {
    type Target = SharedContainer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SharedContainerContainingType {
    // / Tries to wrap a [SharedContainer] into a [SharedContainerContainingType]
    // / Returns if the constraint for [SharedContainerContainingType]] is not satisfied
    // / (i.e. the allowed type of the container is not a [StructuralTypeDefinition::Type])
    // pub fn try_new(container: SharedContainer) -> Result<Self, ()> {
    //     // allowed type of container must only be "shared"
    //     if container
    //         .base_shared_container()
    //         .allowed_type
    //         .with_collapsed_structural_type_definition(|allowed_type| {
    //             !matches!(allowed_type, TypeDefinition::Shared(_))
    //         })
    //     {
    //         return Err(());
    //     }

    //     Ok(SharedContainerContainingType(container))
    // }

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
