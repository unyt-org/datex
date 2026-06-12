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
        ReferenceMutability, ReferencedSharedContainer, SharedContainer,
        SharedContainerMutability, SharedContainerOwnership,
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

impl DatexValueContainerProxyInfallibleSerialize for ReferencedSharedContainer {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(SharedContainer::Referenced(self))
    }
}
impl DatexValueContainerProxySerialize for ReferencedSharedContainer {
    fn try_to_value_container(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.to_value_container())
    }
}
impl DatexValueContainerProxyDeserialize for ReferencedSharedContainer {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        Ok(match value {
            ValueContainer::Shared(SharedContainer::Referenced(reference)) => {
                reference
            }
            _ => return Err(TryFromDatexValueError("Expected ValueContainer::Shared(SharedContainer::Referenced), got something else".to_string())),
        })
    }
}

impl DatexProxyTypes for ReferencedSharedContainer {
    fn datex_type(_memory: &mut Memory) -> Type {
        Type::Alias(TypeDefinitionWithMetadata {
            definition: TypeDefinition::CoreType(
                CoreLibBaseTypeId::Unknown.into(),
            ),
            // TODO
            metadata: TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Referenced(
                    ReferenceMutability::Immutable,
                ),
            },
            reference_name: None,
        })
    }
}

impl DatexValueContainerProxy for ReferencedSharedContainer {}
