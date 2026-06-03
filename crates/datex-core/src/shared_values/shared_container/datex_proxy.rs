use crate::{
    datex_proxy::{
        DatexProxyTypes, DatexValueContainerProxy,
        DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError,
    },
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    runtime::memory::Memory,
    shared_values::{
        ReferenceMutability, SharedContainer, SharedContainerMutability,
        SharedContainerOwnership,
    },
    types::{
        r#type::Type,
        type_definition::TypeDefinition,
        type_definition_with_metadata::{
            TypeDefinitionWithMetadata, TypeMetadata,
        },
    },
    values::value_container::ValueContainer,
};

impl DatexValueContainerProxyInfallibleSerialize for SharedContainer {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(self)
    }
}
impl DatexValueContainerProxySerialize for SharedContainer {
    fn try_to_value_container(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.to_value_container())
    }
}
impl DatexValueContainerProxyDeserialize for SharedContainer {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        Ok(match value {
            ValueContainer::Shared(v) => v,
            _ => return Err(TryFromDatexValueError(
                "Expected ValueContainer::Shared, got ValueContainer::Local"
                    .to_string(),
            )),
        })
    }
}

impl DatexProxyTypes for SharedContainer {
    fn datex_type(_memory: &mut Memory) -> Type {
        Type::Alias(TypeDefinitionWithMetadata {
            definition: TypeDefinition::CoreType(CoreLibBaseTypeId::Unknown.into()),
            // TODO
            metadata: TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Referenced(
                    ReferenceMutability::Immutable,
                ),
            },
        })
    }
}
impl DatexValueContainerProxy for SharedContainer {}
