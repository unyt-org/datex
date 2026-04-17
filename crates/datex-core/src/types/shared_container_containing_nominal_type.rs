use core::ops::Deref;

use crate::{
    libs::core::{
        core_lib_id::CoreLibId,
        type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    },
    shared_values::{
        pointer_address::PointerAddress, shared_containers::SharedContainer,
    },
    types::{
        nominal_type_definition::NominalTypeDefinition, r#type::Type,
        type_match::TypeMatch,
    },
    values::core_value::CoreValue,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SharedContainerContainingNominalType(SharedContainer);

impl Deref for SharedContainerContainingNominalType {
    type Target = SharedContainer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SharedContainerContainingNominalType {
    /// Creates a new [SharedContainerContainingNominalType] from a [SharedContainer] without checking the constraint.
    /// The caller must ensure that the constraint for [SharedContainerContainingNominalType] is satisfied
    /// (i.e. the allowed type of the container is a [StructuralTypeDefinition::NominalType])
    pub unsafe fn new_unchecked(container: SharedContainer) -> Self {
        SharedContainerContainingNominalType(container)
    }

    /// Calls the provided callback with a reference to the recursively collapsed inner [NominalTypeDefinition] value of the shared container
    /// The [SharedContainerContainingNominalType] guarantees that the inner value is always a [CoreValue::NominalType], so this method can never panic.
    pub fn with_collapsed_definition<R>(
        &self,
        f: impl FnOnce(&NominalTypeDefinition) -> R,
    ) -> R {
        self.0.with_collapsed_value(|value| match &value.inner {
            CoreValue::NominalType(ty) => f(ty),
            _ => unreachable!("The constraint for SharedContainerContainingNominalType guarantees that the inner value is always a CoreValue::NominalType")
        })
    }

    /// Tries to get the [CoreLibTypeId] of the inner type of the shared container, if it is a core library type
    pub fn try_get_core_lib_type_id(&self) -> Option<CoreLibTypeId> {
        match CoreLibId::try_from(self.0.pointer_address()).ok()? {
            CoreLibId::Type(ty) => Some(ty),
            _ => None,
        }
    }
}

impl TypeMatch for SharedContainerContainingNominalType {
    fn matches(&self, definition: &Self) -> bool {
        // if it is directly the same nominal type definition
        if self.pointer_address() == definition.pointer_address() {
            return true;
        }
        // if we are a subvariant of the nominal type definition, no recursion
        self.with_collapsed_definition(
            |inner_definition| match inner_definition {
                NominalTypeDefinition::Variant { base, .. } => {
                    base.pointer_address() == definition.pointer_address()
                }
                _ => false,
            },
        )
    }
}
