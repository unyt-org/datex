//! Implements [DatexValueProxy] for [IndexMap<K, V, RandomState>] where K: [DatexValueProxy] + Eq + Hash and V: [DatexValueProxy].

pub mod classification;
mod convert_parts;
mod convert_value;
mod datex_hash;
mod datex_native;
mod datex_native_structural;
mod get_core_lib_type_id;
pub mod get_datex_type;
mod iter_parts;
mod serde_dif;
#[cfg(feature = "ast")]
mod to_datex_expression_data;
mod to_instructions;
mod update_handler;
mod value_access;

#[cfg(test)]
mod tests {
    use crate::{
        traits::get_datex_type::GetDatexType,
        values::{
            core_values::{endpoint::Endpoint, integer::Integer, map::Map},
            value::{Value, value_classification::ValueClassification},
            value_container::ValueContainer,
        },
    };
    use indexmap::IndexMap;

    #[test]
    #[cfg(feature = "std")]
    fn to_value() {
        let mut index_map = IndexMap::new();
        index_map.insert(Integer::from(1), Endpoint::new("@jonas"));
        let index_map_clone = index_map.clone();
        let value = Value::new(index_map);
        assert_eq!(
            value
                .try_into_value::<IndexMap<Integer, Endpoint>>()
                .unwrap(),
            index_map_clone,
        );
    }

    #[test]
    #[allow(clippy::mutable_key_type)]
    #[cfg(feature = "std")]
    fn from_value() {
        use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

        let cache = &mut SharedReferencesCache::default();
        // map with [Value], [Value] as key and value
        let mut map = IndexMap::new();
        map.insert(
            Value::from(Integer::from(1)),
            Value::from(Endpoint::new("@jonas")),
        );
        let value: Value = Value::native(map.clone());
        let map_from_value: IndexMap<Value, Value> =
            value.try_into_value().unwrap();
        assert_eq!(map, map_from_value);

        // map with [ValueContainer], [ValueContainer] as key and value
        let mut map = IndexMap::new();
        map.insert(
            ValueContainer::from(Integer::from(1)),
            ValueContainer::from(Endpoint::new("@jonas")),
        );
        let value: Value = Value::native(map.clone());
        let map_from_value = value
            .try_into_value::<IndexMap<ValueContainer, ValueContainer>>()
            .unwrap();
        assert_eq!(map, map_from_value);

        // map with [Integer, Endpoint] as key and value
        let mut map = IndexMap::new();
        map.insert(Integer::from(1), Endpoint::new("@jonas"));
        let value: Value = Value::native(map.clone());
        let map_from_value = value
            .try_into_value::<IndexMap<Integer, Endpoint>>()
            .unwrap();
        assert_eq!(map, map_from_value);
    }

    #[test]
    #[cfg(feature = "std")]
    fn datex_type() {
        use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

        let map_type = IndexMap::<Integer, Endpoint>::datex_type(
            &mut SharedReferencesCache::default(),
        );
        map_type.with_collapsed_type_definition(|d| {
            use crate::types::type_definition::{
                TypeDefinition,
                collection::{
                    CollectionTypeDefinition,
                    type_definition::map::MapCollectionTypeDefinition,
                },
            };

            assert_eq!(
                d,
                &TypeDefinition::Collection(CollectionTypeDefinition::Map(
                    MapCollectionTypeDefinition::new(
                        Integer::datex_type(
                            &mut SharedReferencesCache::default()
                        ),
                        Endpoint::datex_type(
                            &mut SharedReferencesCache::default()
                        ),
                    )
                ))
            )
        });
    }
}
