use crate::{
    preludes::derive::{SharedReferencesCache, Type, TypeDefinition},
    random::RandomState,
    traits::get_datex_type::GetDatexType,
    types::type_definition::collection::{
        CollectionTypeDefinition,
        type_definition::map::MapCollectionTypeDefinition,
    },
};
use core::hash::Hash;
use indexmap::IndexMap;

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
