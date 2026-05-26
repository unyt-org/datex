use crate::{
    datex_proxy::{
        DatexValueContainerProxy, DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError,
    },
    prelude::*,
    shared_values::{ReferencedSharedContainer, SharedContainer},
    values::value_container::ValueContainer,
};
use crate::datex_proxy::DatexProxyTypes;
use crate::libs::core::type_id::CoreLibBaseTypeId;
use crate::runtime::memory::Memory;
use crate::shared_values::{ReferenceMutability, SharedContainerMutability, SharedContainerOwnership};
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;
use crate::types::type_definition_with_metadata::{TypeDefinitionWithMetadata, TypeMetadata};

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
    fn datex_type(memory: &mut Memory) -> Type {
        Type::Alias(TypeDefinitionWithMetadata {
            definition: TypeDefinition::Core(CoreLibBaseTypeId::Unknown.into()),
            // TODO
            metadata: TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Referenced(ReferenceMutability::Immutable),
            }
        })
    }
}

impl DatexValueContainerProxy for ReferencedSharedContainer {}
