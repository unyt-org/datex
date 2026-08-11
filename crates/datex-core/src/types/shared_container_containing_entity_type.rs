use crate::{
    prelude::*,
    runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
    shared_values::{
        SharedContainer, SharedContainerMutability,
        traits::SharedContainerCommon,
    },
    types::{
        entities::entity_type_definition::EntityTypeDefinition,
        shared_container_containing_type::SharedContainerContainingType,
        traits::type_match::{TypeSatisfiesValueContainer, TypeSuperset},
    },
    values::{core_value::CoreValue, value_container::ValueContainer},
};
use core::{cell::Ref, ops::Deref};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SharedContainerContainingEntityType(SharedContainer);

impl Deref for SharedContainerContainingEntityType {
    type Target = SharedContainer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<SharedContainerContainingEntityType> for SharedContainer {
    fn from(value: SharedContainerContainingEntityType) -> Self {
        value.0
    }
}

impl From<SharedContainerContainingEntityType>
    for SharedContainerContainingType
{
    fn from(value: SharedContainerContainingEntityType) -> Self {
        unsafe { SharedContainerContainingType::new_unchecked(value.0) }
    }
}

impl SharedContainerContainingEntityType {
    pub fn new_from_definition(
        definition: EntityTypeDefinition,
        address_provider: &mut SelfOwnedPointerAddressProvider,
    ) -> SharedContainerContainingEntityType {
        SharedContainerContainingEntityType(
            SharedContainer::new_owned_with_inferred_allowed_type(
                CoreValue::EntityTypeDefinition(definition),
                SharedContainerMutability::Immutable,
                address_provider,
            ),
        )
    }

    /// Converts the [SharedContainerContainingEntityType] into a [SharedContainer], consuming the wrapper.
    pub fn to_shared_container(self) -> SharedContainer {
        self.0
    }

    /// Creates a new [SharedContainerContainingEntityType] from a [SharedContainer] without checking the constraint.
    /// # Safety
    /// The caller must ensure that the constraint for [SharedContainerContainingEntityType] is satisfied
    /// (i.e. the allowed type of the container is a [Type::Nominal])
    pub unsafe fn new_unchecked(container: SharedContainer) -> Self {
        SharedContainerContainingEntityType(container)
    }

    /// Returns a reference to the inner [EntityTypeDefinition] contained in the [SharedContainer].
    /// The [SharedContainerContainingEntityType] guarantees that the inner value is always a [CoreValue::EntityTypeDefinition], so this method can never panic.
    pub fn entity_definition(&self) -> Ref<'_, EntityTypeDefinition> {
        let val = self.0.value_container();
        Ref::map(val, |v| match v.try_as::<EntityTypeDefinition>() {
            Some(ty) => ty,
            _ => unreachable!(
                "The constraint for SharedContainerContainingEntityType guarantees that the inner value is always a CoreValue::EntityTypeDefinition"
            ),
        })
    }
}

impl TryFrom<SharedContainer> for SharedContainerContainingEntityType {
    type Error = ();
    fn try_from(value: SharedContainer) -> Result<Self, Self::Error> {
        // container must be immutable and contain nominal type
        if value.container_mutability() == SharedContainerMutability::Immutable
        {
            let is_nominal = {
                let val = value.collapsed_value();
                let val_sheep = val.borrow();
                matches!(&val_sheep.inner, CoreValue::EntityTypeDefinition(_))
            };

            if is_nominal {
                Ok(SharedContainerContainingEntityType(value))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

impl TypeSuperset<SharedContainerContainingEntityType>
    for SharedContainerContainingEntityType
{
    fn is_superset_of(
        &self,
        other: &SharedContainerContainingEntityType,
    ) -> bool {
        // if it is directly the same nominal type definition
        self.pointer_address() == other.pointer_address()
    }
}

impl TypeSatisfiesValueContainer for SharedContainerContainingEntityType {
    fn satisfies_value_container(&self, _value: &ValueContainer) -> bool {
        todo!()
    }
}
