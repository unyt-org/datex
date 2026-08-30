//! Implements [DatexValueProxy] for [HashMap<K, V>] where K: [DatexValueProxy] + Eq + Hash and V: [DatexValueProxy].

#[cfg(feature = "decompiler")]
mod to_datex_expression_data;
mod value_access;
pub mod get_datex_type;
mod datex_native;
mod datex_native_only_structural;

use crate::{
    collections::HashMap,
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
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
    use crate::traits::get_datex_type::GetDatexType;
    use super::*;
    use crate::values::{
        core_value::CoreValue,
        core_values::{endpoint::Endpoint, integer::Integer},
        value::Value,
        value_container::ValueContainer,
    };
    #[test]
    fn to_value() {
        let mut map = HashMap::new();
        map.insert(Integer::from(1), Endpoint::new("@jonas"));
        let value: Value = Value::native_only_structural(map);
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
    fn from_value() {
        // map with [Value], [Value] as key and value
        let mut map = HashMap::new();
        map.insert(
            Value::from(Integer::from(1)),
            Value::from(Endpoint::new("@jonas")),
        );
        let value: Value = Value::native_only_structural(map.clone());
        let map_from_value: HashMap<Value, Value> = value.try_into_value().unwrap();
        assert_eq!(map, map_from_value);

        // map with [ValueContainer], [ValueContainer] as key and value
        let mut map = HashMap::new();
        map.insert(
            ValueContainer::from(Integer::from(1)),
            ValueContainer::from(Endpoint::new("@jonas")),
        );
        let value: Value = Value::native_only_structural(map.clone());
        let map_from_value =
            HashMap::<ValueContainer, ValueContainer>::try_from_value(value)
                .unwrap();
        assert_eq!(map, map_from_value);

        // map with [Integer, Endpoint] as key and value
        let mut map = HashMap::new();
        map.insert(Integer::from(1), Endpoint::new("@jonas"));
        let value: Value = Value::native_only_structural(map.clone());
        let map_from_value =
            HashMap::<Integer, Endpoint>::try_from_value(value).unwrap();
        assert_eq!(map, map_from_value);
    }

    #[test]
    fn datex_type() {
        let map_type = HashMap::<Integer, Endpoint>::datex_type(&mut SharedReferencesCache::default());
        map_type.with_collapsed_type_definition(|d| {
            assert_eq!(
                d,
                &TypeDefinition::Collection(CollectionTypeDefinition::Map(
                    MapCollectionTypeDefinition::new(
                        Integer::datex_type(&mut SharedReferencesCache::default()),
                        Endpoint::datex_type(&mut SharedReferencesCache::default()),
                    )
                ))
            )
        });
    }
}
