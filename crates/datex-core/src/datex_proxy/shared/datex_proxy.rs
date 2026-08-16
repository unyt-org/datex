use crate::{
    datex_proxy::{
        DatexProxyTypes, DatexValueContainerProxy,
        DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerde, DatexValueContainerProxySerialize,
        DatexValueProxy, DatexValueProxyInfallibleSerialize,
        DatexValueProxySerialize, TryFromDatexValueError, TryToDatexValueError,
        shared::Shared,
    },
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
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
impl<T, C> DatexValueContainerProxyInfallibleSerialize<C> for Shared<T, C>
where
    Shared<T, C>: DatexValueProxyInfallibleSerialize<C>,
    T: DatexValueContainerProxySerde<C>,
{
    fn to_value_container(self, _context: &mut C) -> ValueContainer {
        // FIXME
        ValueContainer::Shared(self.container)
    }
}

impl<T, C> DatexValueContainerProxySerialize<C> for Shared<T, C>
where
    Shared<T, C>: DatexValueProxySerialize<C>,
    T: DatexValueContainerProxySerde<C>,
{
    fn try_to_value_container(
        self,
        _context: &mut C,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(ValueContainer::Shared(self.container))
    }
}

impl<T, C> DatexValueContainerProxyDeserialize for Shared<T, C>
where
    T: DatexValueContainerProxySerde<C>,
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

impl<T, C> DatexProxyTypes<C> for Shared<T, C>
where
    T: DatexValueContainerProxySerde<C> + DatexProxyTypes<C>,
{
    fn datex_type(context: &mut C) -> Type {
        Type::Definition(TypeDefinitionWithMetadata::new(
            TypeDefinition::Nested(Box::new(T::datex_type(context))),
            TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Owned,
            },
        ))
    }
}

impl<T, C> DatexValueContainerProxy<C> for Shared<T, C>
where
    Shared<T, C>: DatexValueContainerProxy<C>,
    T: DatexValueContainerProxySerde<C>,
{
}
