use crate::shared_wrappers::shared::Shared;
use crate::values::core_values::native::DatexNative;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::shared_values::{SharedContainerMutability, SharedContainerOwnership};
use crate::traits::get_datex_type::GetDatexType;
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;
use crate::types::type_definition_with_metadata::{TypeDefinitionWithMetadata, TypeMetadata};

impl<T> GetDatexType for Shared<T>
where
    T: DatexNative + GetDatexType,
{
    fn datex_type(context: &mut SharedReferencesCache) -> Type {
        Type::Definition(TypeDefinitionWithMetadata::new(
            TypeDefinition::Box(Box::new(T::datex_type(context))),
            TypeMetadata::Shared {
                mutability: SharedContainerMutability::Mutable,
                ownership: SharedContainerOwnership::Owned,
            },
        ))
    }
}