use crate::{
    libs::core::{core_lib_id::CoreLibId, type_id::CoreLibTypeId},
    runtime::{
        memory::Memory,
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::{SharedContainer, SharedContainerMutability},
    types::{
        nominal_type_definition::NominalTypeDefinition,
        shared_container_containing_type::SharedContainerContainingType,
        type_match::TypeMatch,
    },
    values::{core_value::CoreValue, value_container::ValueContainer},
};
use core::ops::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize)]
pub struct SharedContainerContainingNominalType(SharedContainer);

impl Deref for SharedContainerContainingNominalType {
    type Target = SharedContainer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<SharedContainerContainingNominalType> for SharedContainer {
    fn from(value: SharedContainerContainingNominalType) -> Self {
        value.0
    }
}

impl From<SharedContainerContainingNominalType>
    for SharedContainerContainingType
{
    fn from(value: SharedContainerContainingNominalType) -> Self {
        unsafe { SharedContainerContainingType::new_unchecked(value.0) }
    }
}

impl SharedContainerContainingNominalType {
    pub fn new_from_definition(
        definition: NominalTypeDefinition,
        address_provider: &mut SelfOwnedPointerAddressProvider,
        memory: &Memory,
    ) -> SharedContainerContainingNominalType {
        SharedContainerContainingNominalType(
            SharedContainer::new_owned_with_inferred_allowed_type(
                CoreValue::NominalTypeDefinition(definition),
                SharedContainerMutability::Immutable,
                address_provider,
                memory,
            ),
        )
    }

    /// Creates a new [SharedContainerContainingNominalType] from a [SharedContainer] without checking the constraint.
    /// The caller must ensure that the constraint for [SharedContainerContainingNominalType] is satisfied
    /// (i.e. the allowed type of the container is a [StructuralTypeDefinition::NominalType])
    pub unsafe fn new_unchecked(container: SharedContainer) -> Self {
        SharedContainerContainingNominalType(container)
    }

    /// Calls the provided callback with a reference to the recursively collapsed inner [NominalTypeDefinition] value of the shared container
    /// The [SharedContainerContainingNominalType] guarantees that the inner value is always a [CoreValue::NominalTypeDefinition], so this method can never panic.
    pub fn with_collapsed_definition<R>(
        &self,
        f: impl FnOnce(&NominalTypeDefinition) -> R,
    ) -> R {
        self.0.with_collapsed_value(|value| match &value.inner {
            CoreValue::NominalTypeDefinition(ty) => f(ty),
            _ => unreachable!("The constraint for SharedContainerContainingNominalType guarantees that the inner value is always a CoreValue::NominalType")
        })
    }

    /// Tries to get the [CoreLibTypeId] of the inner type of the shared container, if it is a core library type
    pub fn try_get_core_lib_type_id(&self) -> Option<CoreLibTypeId> {
        match CoreLibId::try_from(&self.0.pointer_address()).ok()? {
            CoreLibId::Type(ty) => Some(ty),
            _ => None,
        }
    }
}

impl TryFrom<SharedContainer> for SharedContainerContainingNominalType {
    type Error = ();
    fn try_from(value: SharedContainer) -> Result<Self, Self::Error> {
        // container must be immutable and contain nominal type
        if value.container_mutability() == SharedContainerMutability::Immutable
        {
            if value.with_collapsed_value_mut(|v| match &v.inner {
                CoreValue::NominalTypeDefinition(_) => true,
                _ => false,
            }) {
                Ok(SharedContainerContainingNominalType(value))
            } else {
                Err(())
            }
        } else {
            Err(())
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

    fn matched_by_value(&self, _value: &ValueContainer) -> bool {
        todo!()
    }
}
