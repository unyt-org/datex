//! Implements [DatexValueProxy] for [HashMap<K, V>] where K: [DatexValueProxy] + Eq + Hash and V: [DatexValueProxy].
use crate::{
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
};
use core::hash::Hash;
use std::any::Any;
use crate::ast::expressions::DatexExpressionData;
use crate::ast::spanned::Spanned;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::native::DatexNative;

impl<
    K: DatexValueContainerProxy + Eq + Hash + 'static,
    V: DatexValueContainerProxy + 'static,
> DatexValueProxy for HashMap<K, V>
{
}

impl<
    K: DatexValueContainerProxyDeserialize + Eq + Hash + 'static,
    V: DatexValueContainerProxyDeserialize + 'static,
> DatexValueProxyDeserialize for HashMap<K, V>
{
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        match Map::try_from(value) {
            Ok(map) => map
                .into_iter()
                .map(|(k, v)| {
                    let key = K::try_from_value_container(k.into())?;
                    let value = V::try_from_value_container(v)?;
                    Ok((key, value))
                })
                .collect::<Result<HashMap<K, V>, _>>(),
            Err(e) => Err(e),
        }
    }
}

impl<
    K: DatexValueContainerProxySerialize + Eq + Hash,
    V: DatexValueContainerProxySerialize,
> DatexValueProxySerialize for HashMap<K, V>
{
    fn try_boxed_to_value(
        self: Box<Self>,
        context: &mut SharedReferencesCache,
    ) -> Result<Value, TryToDatexValueError> {
        let map = self
            .into_iter()
            .map(|(k, v)| {
                let key = Box::new(k).try_boxed_to_value_container(context)?;
                let value =
                    Box::new(v).try_boxed_to_value_container(context)?;
                Ok((key, value))
            })
            .collect::<Result<Map, _>>()?;
        Ok(Value::from(map))
    }
}

impl<
    K: DatexValueContainerProxyInfallibleSerialize + Eq + Hash,
    V: DatexValueContainerProxyInfallibleSerialize,
> DatexValueProxyInfallibleSerialize for HashMap<K, V>
{
    fn boxed_to_value(
        self: Box<Self>,
        context: &mut SharedReferencesCache,
    ) -> Value {
        let map = self
            .into_iter()
            .map(|(k, v)| {
                let key = Box::new(k).boxed_to_value_container(context);
                let value = Box::new(v).boxed_to_value_container(context);
                (key, value)
            })
            .collect::<Map>();
        Value::from(map)
    }
}

impl<K, V> DatexProxyType for HashMap<K, V>
where
    K: DatexProxyType + Eq + Hash,
    V: DatexProxyType,
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

impl<K, V> ToDatexExpressionData for HashMap<K, V>
where
    K: ToDatexExpressionData + Eq + Hash,
    V: ToDatexExpressionData,
{
    fn to_datex_expression_data(
        &self,
    ) -> DatexExpressionData {
        DatexExpressionData::Map(
            self.iter()
                .map(|(k, v)| {
                    (
                        k.to_datex_expression_data().with_default_span(),
                        v.to_datex_expression_data().with_default_span(),
                    )
                })
                .collect(),
        )
    }
}

// TODO: clean up traits
impl<K, V> DatexNative for HashMap<K, V>
where
    K: DatexNative + DatexProxyType + DatexValueProxy + Eq + Hash,
    V: DatexNative + DatexProxyType + DatexValueProxy,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn boxed_to_datex_native_value(self: Box<Self>, cache: &mut SharedReferencesCache) -> Value {
        Value::native_boxed(self, cache)
    }
}


#[cfg(test)]
mod tests {
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
        let value: Value = map.to_value_without_cache();
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
        let value: Value = map.clone().to_value_without_cache();
        let map_from_value =
            HashMap::<Value, Value>::try_from_value(value).unwrap();
        assert_eq!(map, map_from_value);

        // map with [ValueContainer], [ValueContainer] as key and value
        let mut map = HashMap::new();
        map.insert(
            ValueContainer::from(Integer::from(1)),
            ValueContainer::from(Endpoint::new("@jonas")),
        );
        let value: Value = map.clone().to_value_without_cache();
        let map_from_value =
            HashMap::<ValueContainer, ValueContainer>::try_from_value(value)
                .unwrap();
        assert_eq!(map, map_from_value);

        // map with [Integer, Endpoint] as key and value
        let mut map = HashMap::new();
        map.insert(Integer::from(1), Endpoint::new("@jonas"));
        let value: Value = map.clone().to_value_without_cache();
        let map_from_value =
            HashMap::<Integer, Endpoint>::try_from_value(value).unwrap();
        assert_eq!(map, map_from_value);
    }

    #[test]
    fn datex_type() {
        let map_type = HashMap::<Integer, Endpoint>::datex_type_without_cache();
        map_type.with_collapsed_type_definition(|d| {
            assert_eq!(
                d,
                &TypeDefinition::Collection(CollectionTypeDefinition::Map(
                    MapCollectionTypeDefinition::new(
                        Integer::datex_type_without_cache(),
                        Endpoint::datex_type_without_cache(),
                    )
                ))
            )
        });
    }
}
