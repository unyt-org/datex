use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::{
        ReferenceMutability, ReferencedSharedContainer,
        SharedContainerMutability, SharedContainerOwnership,
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

impl GetDatexType for ReferencedSharedContainer {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        // TODO get the mutability and ownership information, do we want to use an union here
        // or have a unspecified reference metadata here?

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
