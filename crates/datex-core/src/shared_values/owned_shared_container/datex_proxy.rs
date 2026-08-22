use crate::{
    datex_proxy::{
        DatexProxyType, DatexValueContainerProxy,
        DatexValueContainerProxyDeserialize,
        DatexValueContainerProxyInfallibleSerialize,
        DatexValueContainerProxySerialize, TryFromDatexValueError,
        TryToDatexValueError,
    },
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::{
        OwnedSharedContainer, SharedContainer, SharedContainerMutability,
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

impl DatexValueContainerProxyInfallibleSerialize for OwnedSharedContainer {
    fn boxed_to_value_container(
        self: Box<Self>,
        _context: &mut SharedReferencesCache,
    ) -> ValueContainer {
        ValueContainer::Shared(SharedContainer::Owned(*self))
    }
}
impl DatexValueContainerProxySerialize for OwnedSharedContainer {
    fn try_boxed_to_value_container(
        self: Box<Self>,
        context: &mut SharedReferencesCache,
    ) -> Result<ValueContainer, TryToDatexValueError> {
        Ok(self.boxed_to_value_container(context))
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

impl DatexProxyType for OwnedSharedContainer {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::Definition(TypeDefinitionWithMetadata::new(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Any.into()),
            // TODO
            TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Owned,
            },
        ))
    }
}

impl DatexValueContainerProxy for OwnedSharedContainer {}
