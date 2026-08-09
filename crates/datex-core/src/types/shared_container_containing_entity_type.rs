use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
    shared_values::{
        SelfOwnedPointerAddress, SharedContainer, SharedContainerMutability,
        traits::SharedContainerCommon,
    },
    types::{
        entity_type_definition::EntityTypeDefinition,
        shared_container_containing_type::SharedContainerContainingType,
        traits::type_match::{TypeSatisfiesValueContainer, TypeSuperset},
        r#type::Type,
    },
    values::{core_value::CoreValue, value_container::ValueContainer},
};
use core::ops::Deref;

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

    /// Creates a new [SharedContainerContainingEntityType] with the
    /// given address, type and name.
    /// # Safety
    /// The caller must ensure that the address is not used anywhere else.
    pub unsafe fn new_base_with_address(
        name: String,
        address: SelfOwnedPointerAddress,
        ty: Type,
    ) -> SharedContainerContainingEntityType {
        unsafe {
            SharedContainerContainingEntityType::new_unchecked(
                SharedContainer::new_owned_with_inferred_allowed_type_unsafe(
                    CoreValue::EntityTypeDefinition(
                        EntityTypeDefinition::new_base(ty, name),
                    ),
                    SharedContainerMutability::Immutable,
                    address,
                ),
            )
        }
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

    /// Calls the provided callback with a reference to the recursively collapsed inner [EntityTypeDefinition] value of the shared container
    /// The [SharedContainerContainingEntityType] guarantees that the inner value is always a [CoreValue::EntityTypeDefinition], so this method can never panic.
    pub fn with_collapsed_definition<R>(
        &self,
        f: impl FnOnce(&EntityTypeDefinition) -> R,
    ) -> R {
        let val = self.0.collapsed_value();
        let val_sheep = val.borrow();
        let ty = match &val_sheep.inner {
            CoreValue::EntityTypeDefinition(ty) => ty,
            _ => unreachable!(
                "The constraint for SharedContainerContainingNominalType guarantees that the inner value is always a CoreValue::NominalType"
            ),
        };
        f(ty)
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
        if self.pointer_address() == other.pointer_address() {
            return true;
        }
        // if other is a subvariant of the nominal type definition, no recursion
        other.with_collapsed_definition(|inner_definition| {
            match inner_definition {
                EntityTypeDefinition::Variant { base, .. } => {
                    base.pointer_address() == self.pointer_address()
                }
                _ => false,
            }
        })
    }
}

impl TypeSatisfiesValueContainer for SharedContainerContainingEntityType {
    fn satisfies_value_container(&self, _value: &ValueContainer) -> bool {
        todo!()
    }
}
