use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
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
};
use crate::traits::get_datex_type::GetDatexType;

impl GetDatexType for SharedContainer {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::Definition(TypeDefinitionWithMetadata::new(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Any.into()),
            TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Referenced(
                    ReferenceMutability::Immutable,
                ),
            },
        ))
    }
}
