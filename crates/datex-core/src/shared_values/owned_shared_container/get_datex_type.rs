use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::{
        OwnedSharedContainer, SharedContainerMutability,
        SharedContainerOwnership,
    },
    traits::get_datex_type::GetDatexType,
    types::{
        r#type::Type,
        type_definition::TypeDefinition,
        type_definition_with_metadata::{
            TypeDefinitionWithMetadata, TypeMetadata,
        },
    },
};

impl GetDatexType for OwnedSharedContainer {
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
