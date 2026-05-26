use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError, shared::Shared,
    },
    prelude::*,
    values::value_container::ValueContainer,
};
use crate::datex_proxy::DatexProxyTypes;
use crate::runtime::memory::Memory;
use crate::shared_values::{SharedContainerMutability, SharedContainerOwnership};
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;
use crate::types::type_definition_with_metadata::{TypeDefinitionWithMetadata, TypeMetadata};

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
    fn datex_type(memory: &mut Memory) -> Type {
        Type::Alias(TypeDefinitionWithMetadata {
            definition: TypeDefinition::Nested(Box::new(T::datex_type(memory))),
            metadata: TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Owned,
            }
        })
    }
}

impl<T: DatexValueContainerProxy> DatexValueContainerProxy for Shared<T> {}
