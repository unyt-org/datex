use crate::{
    runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
    shared_values::{
        SharedContainer, SharedContainerMutability,
        traits::SharedContainerCommon,
    },
    types::{
        nominal_type_definition::NominalTypeDefinition,
        shared_container_containing_type::SharedContainerContainingType,
        traits::type_match::{TypeSatisfiesValueContainer, TypeSuperset},
    },
    values::{core_value::CoreValue, value_container::ValueContainer},
};
use core::ops::Deref;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
    ) -> SharedContainerContainingNominalType {
        SharedContainerContainingNominalType(
            SharedContainer::new_owned_with_inferred_allowed_type(
                CoreValue::NominalTypeDefinition(definition),
                SharedContainerMutability::Immutable,
                address_provider,
            ),
        )
    }

    /// Converts the [SharedContainerContainingNominalType] into a [SharedContainer], consuming the wrapper.
    pub fn to_shared_container(self) -> SharedContainer {
        self.0
    }

    /// Creates a new [SharedContainerContainingNominalType] from a [SharedContainer] without checking the constraint.
    /// # Safety
    /// The caller must ensure that the constraint for [SharedContainerContainingNominalType] is satisfied
    /// (i.e. the allowed type of the container is a [Type::Nominal])
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

impl TypeSuperset<SharedContainerContainingNominalType>
    for SharedContainerContainingNominalType
{
    fn is_superset_of(
        &self,
        other: &SharedContainerContainingNominalType,
    ) -> bool {
        // if it is directly the same nominal type definition
        if self.pointer_address() == other.pointer_address() {
            return true;
        }
        // if other is a subvariant of the nominal type definition, no recursion
        other.with_collapsed_definition(|inner_definition| {
            match inner_definition {
                NominalTypeDefinition::Variant { base, .. } => {
                    base.pointer_address() == self.pointer_address()
                }
                _ => false,
            }
        })
    }
}

impl TypeSatisfiesValueContainer for SharedContainerContainingNominalType {
    fn satisfies_value_container(&self, _value: &ValueContainer) -> bool {
        todo!()
    }
}
