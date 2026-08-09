use crate::{
    datex_proxy::{
        DatexProxyTypes, DatexValueContainerProxy,
        DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError, shared::Shared,
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

impl<T: DatexValueContainerProxy> DatexValueContainerProxyInfallibleSerialize
    for Shared<T>
{
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(self.container)
    }
}
impl<T: DatexValueContainerProxy> DatexValueContainerProxySerialize
    for Shared<T>
{
    fn try_to_value_container(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.to_value_container())
    }
}
impl<T: DatexValueContainerProxy> DatexValueContainerProxyDeserialize
    for Shared<T>
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

impl<T: DatexValueContainerProxy> DatexProxyTypes for Shared<T> {
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        Type::Definition(TypeDefinitionWithMetadata::new(
            TypeDefinition::Nested(Box::new(T::datex_type(memory))),
            TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Owned,
            },
        ))
    }
}

impl<T: DatexValueContainerProxy> DatexValueContainerProxy for Shared<T> {}
