use core::hash::Hash;
use indexmap::IndexMap;
use crate::preludes::derive::{SharedReferencesCache, Type, TypeDefinition};
use crate::traits::get_datex_type::GetDatexType;
use crate::types::type_definition::collection::CollectionTypeDefinition;
use crate::types::type_definition::collection::type_definition::map::MapCollectionTypeDefinition;
use crate::random::RandomState;

impl<K, V> GetDatexType for IndexMap<K, V, RandomState>
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
