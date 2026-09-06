//! Implements [DatexValueProxy] for [IndexMap<K, V, RandomState>] where K: [DatexValueProxy] + Eq + Hash and V: [DatexValueProxy].

pub mod classification;
mod convert_parts;
mod datex_hash;
mod datex_native;
mod datex_native_structural;
mod get_core_lib_type_id;
pub mod get_datex_type;
#[cfg(feature = "ast")]
mod to_datex_expression_data;
mod to_instructions;
mod try_from_core_value;
mod value_access;
use crate::{
    prelude::*,
    types::{
        r#type::Type,
        type_definition::collection::{
            CollectionTypeDefinition,
            type_definition::map::MapCollectionTypeDefinition,
        },
    },
    values::{core_values::map::Map, value::Value},
};

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::TypeDefinition,
    values::core_values::native::DatexNative,
};
use core::{any::Any, hash::Hash};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        traits::get_datex_type::GetDatexType,
        values::{
            core_value::CoreValue,
            core_values::{endpoint::Endpoint, integer::Integer},
            value::Value,
            value_container::ValueContainer,
        },
    };
    use indexmap::IndexMap;

    #[test]
    #[cfg(feature = "std")]
    fn to_value() {
        let mut map = IndexMap::new();
        map.insert(Integer::from(1), Endpoint::new("@jonas"));
        let value: Value = Value::native_structural(map);
        assert_eq!(
            value.inner,
            CoreValue::Map(Map::from_iter(vec![(
                Integer::from(1),
                Endpoint::new("@jonas")
            )]))
        );
    }

    #[test]
    #[allow(clippy::mutable_key_type)]
    #[cfg(feature = "std")]
    fn from_value() {
        let cache = &mut SharedReferencesCache::default();
        // map with [Value], [Value] as key and value
        let mut map = IndexMap::new();
        map.insert(
            Value::from(Integer::from(1)),
            Value::from(Endpoint::new("@jonas")),
        );
        let value: Value = Value::native(map.clone(), cache);
        let map_from_value: IndexMap<Value, Value> =
            value.try_into_value().unwrap();
        assert_eq!(map, map_from_value);

        // map with [ValueContainer], [ValueContainer] as key and value
        let mut map = IndexMap::new();
        map.insert(
            ValueContainer::from(Integer::from(1)),
            ValueContainer::from(Endpoint::new("@jonas")),
        );
        let value: Value = Value::native(map.clone(), cache);
        let map_from_value = value
            .try_into_value::<IndexMap<ValueContainer, ValueContainer>>()
            .unwrap();
        assert_eq!(map, map_from_value);

        // map with [Integer, Endpoint] as key and value
        let mut map = IndexMap::new();
        map.insert(Integer::from(1), Endpoint::new("@jonas"));
        let value: Value = Value::native(map.clone(), cache);
        let map_from_value = value
            .try_into_value::<IndexMap<Integer, Endpoint>>()
            .unwrap();
        assert_eq!(map, map_from_value);
    }

    #[test]
    #[cfg(feature = "std")]
    fn datex_type() {
        let map_type = IndexMap::<Integer, Endpoint>::datex_type(
            &mut SharedReferencesCache::default(),
        );
        map_type.with_collapsed_type_definition(|d| {
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
