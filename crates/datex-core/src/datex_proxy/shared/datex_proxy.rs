use crate::{
    datex_proxy::{
        DatexProxyTypes, DatexValueContainerProxy,
        DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError, TryToDatexValueError,
        shared::Shared,
    },
    prelude::*,
    shared_values::{SharedContainerMutability, SharedContainerOwnership},
    types::{
        r#type::Type,
        type_definition::TypeDefinition,
        type_definition_with_metadata::{
            TypeDefinitionWithMetadata, TypeMetadata,
        },
    },
    values::value_container::ValueContainer,
};
use crate::datex_proxy::DatexValueProxyInfallibleSerialize;
use crate::values::core_value::DatexNative;

impl<T, C> DatexValueContainerProxyInfallibleSerialize<C> for Shared<T>
where
    Shared<T>: DatexValueProxyInfallibleSerialize<C>,
    T: DatexNative,
{
    fn to_value_container(self, _context: &mut C) -> ValueContainer {
        // FIXME
        ValueContainer::Shared(self.container)
    }
}

impl<T> DatexValueContainerProxySerialize<()> for Shared<T>
where
    T: DatexNative,
{
    fn try_to_value_container(
        self,
        _context: &mut (),
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(ValueContainer::Shared(self.container))
    }
}

impl<T> DatexValueContainerProxyDeserialize for Shared<T>
where
    T: DatexNative,
{
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        match value {
            ValueContainer::Shared(container) => Shared::try_from(container),
            _ => Err(TryFromDatexValueError(
                "Expected ValueContainer::Shared, ValueContainer::Local"
                    .to_string(),
            )),
        }
    }
}

impl<T, C> DatexProxyTypes<C> for Shared<T>
where
    T: DatexNative + DatexProxyTypes<C>,
{
    fn datex_type(context: &mut C) -> Type {
        Type::Definition(TypeDefinitionWithMetadata::new(
            TypeDefinition::Box(Box::new(T::datex_type(context))),
            TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Owned,
            },
        ))
    }
}

impl<T, C> DatexValueContainerProxy<C> for Shared<T>
where
    Shared<T>: DatexValueContainerProxy<C>,
    T: DatexNative,
{
}
