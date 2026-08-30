use crate::collections::HashMap;
use core::hash::Hash;
use crate::preludes::derive::{SharedReferencesCache, Type, TypeDefinition};
use crate::traits::get_datex_type::GetDatexType;
use crate::types::type_definition::collection::CollectionTypeDefinition;
use crate::types::type_definition::collection::type_definition::map::MapCollectionTypeDefinition;

impl<K, V> GetDatexType for HashMap<K, V>
where
    K: GetDatexType + Eq + Hash,
    V: GetDatexType,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::Collection(CollectionTypeDefinition::Map(
                MapCollectionTypeDefinition::new(
                    K::datex_type(memory),
                    V::datex_type(memory),
                ),
            ))
                .into(),
        )
    }
}
