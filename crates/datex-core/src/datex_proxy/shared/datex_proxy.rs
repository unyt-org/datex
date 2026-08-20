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
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::values::core_value::DatexNative;

impl<T> DatexValueContainerProxyInfallibleSerialize for Shared<T>
where
    Shared<T>: DatexValueProxyInfallibleSerialize,
    T: DatexNative,
{
    fn to_value_container(self, _context: &mut SharedReferencesCache) -> ValueContainer {
        // FIXME
        ValueContainer::Shared(self.container)
    }
}

impl<T> DatexValueContainerProxySerialize for Shared<T>
where
    T: DatexNative,
{
    fn try_to_value_container(
        self,
        _context: &mut SharedReferencesCache,
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

impl<T> DatexProxyTypes for Shared<T>
where
    T: DatexNative + DatexProxyTypes,
{
    fn datex_type(context: &mut SharedReferencesCache) -> Type {
        Type::Definition(TypeDefinitionWithMetadata::new(
            TypeDefinition::Box(Box::new(T::datex_instance_type(context))),
            TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Owned,
            },
        ))
    }
}

impl<T> DatexValueContainerProxy for Shared<T>
where
    Shared<T>: DatexValueContainerProxy,
    T: DatexNative,
{
}
