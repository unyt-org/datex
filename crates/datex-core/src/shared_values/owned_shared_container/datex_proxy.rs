use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError,
    },
    prelude::*,
    shared_values::{OwnedSharedContainer, SharedContainer},
    values::value_container::ValueContainer,
};
use crate::datex_proxy::DatexProxyTypes;
use crate::libs::core::type_id::CoreLibBaseTypeId;
use crate::runtime::memory::Memory;
use crate::shared_values::{ReferenceMutability, SharedContainerMutability, SharedContainerOwnership};
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;
use crate::types::type_definition_with_metadata::{TypeDefinitionWithMetadata, TypeMetadata};

impl DatexValueContainerProxyInfallibleSerialize for OwnedSharedContainer {
    fn to_value_container(self) -> ValueContainer {
        ValueContainer::Shared(SharedContainer::Owned(self))
    }
}
impl DatexValueContainerProxySerialize for OwnedSharedContainer {
    fn try_to_value_container(
        self,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.to_value_container())
    }
}
impl DatexValueContainerProxyDeserialize for OwnedSharedContainer {
    fn try_from_value_container(
        value: ValueContainer,
    ) -> Result<Self, TryFromDatexValueError> {
        Ok(match value {
            ValueContainer::Shared(SharedContainer::Owned(owned)) => owned,
            _ => return Err(TryFromDatexValueError("Expected ValueContainer::Shared(SharedContainer::Owned), got something else".to_string())),
        })
    }
}

impl DatexProxyTypes for OwnedSharedContainer {
    fn datex_type(memory: &mut Memory) -> Type {
        Type::Alias(TypeDefinitionWithMetadata {
            definition: TypeDefinition::Core(CoreLibBaseTypeId::Unknown.into()),
            // TODO
            metadata: TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Owned,
            }
        })
    }
}

impl DatexValueContainerProxy for OwnedSharedContainer {}
