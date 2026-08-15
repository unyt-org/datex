use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibVariantTypeId},
    prelude::*,
    shared_values::errors::KeyNotFoundError,
    types::{
        entities::entity_type_definition::EntityTypeDefinition,
        r#type::Type,
        type_definition::collection::{
            CollectionTypeDefinition,
            type_definition::{
                list::ListCollectionTypeDefinition,
                map::MapCollectionTypeDefinition,
            },
        },
    },
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean,
            callable::Callable,
            decimal::{Decimal, typed_decimal::TypedDecimal},
            endpoint::Endpoint,
            integer::{Integer, typed_integer::TypedInteger},
            list::List,
            map::Map,
            range::Range,
            text::Text,
        },
        value::Value,
        value_container::ValueContainer,
    },
};

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::{TypeDefinition, union::UnionTypeDefinition},
    values::core_values::{
        decimal::typed_decimal::DecimalTypeVariant,
        integer::typed_integer::IntegerTypeVariant,
    },
};
use core::hash::Hash;

impl<K: DatexValueContainerProxy + Eq + Hash, V: DatexValueContainerProxy>
    DatexValueProxy for HashMap<K, V>
{
}

impl<K: DatexValueContainerProxy + Eq + Hash, V: DatexValueContainerProxy>
    DatexValueProxyDeserialize for HashMap<K, V>
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

impl<K: DatexValueContainerProxy + Eq + Hash, V: DatexValueContainerProxy>
    DatexValueProxySerialize for HashMap<K, V>
{
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        let map = self
            .into_iter()
            .map(|(k, v)| {
                let key = k.try_to_value_container()?;
                let value = v.try_to_value_container()?;
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
    fn to_value(self) -> Value {
        let map = self
            .into_iter()
            .map(|(k, v)| {
                let key = k.to_value_container();
                let value = v.to_value_container();
                (key, value)
            })
            .collect::<Map>();
        Value::from(map)
    }
}

impl<K, V> DatexProxyTypes for HashMap<K, V>
where
    K: DatexProxyTypes + Eq + Hash,
    V: DatexProxyTypes,
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
