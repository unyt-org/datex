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

impl<T> DatexValueProxy for Box<T> where T: DatexValueProxy {}

impl<T> DatexValueProxySerialize for Box<T>
where
    T: DatexValueProxySerialize,
{
    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
        (*self).try_to_value()
    }
}

impl<T> DatexValueProxyInfallibleSerialize for Box<T>
where
    T: DatexValueProxyInfallibleSerialize,
{
    fn to_value(self) -> Value {
        (*self).to_value()
    }
}

impl<T> DatexValueProxyDeserialize for Box<T>
where
    T: DatexValueProxyDeserialize,
{
    fn try_from_value(value: Value) -> Result<Self, TryFromDatexValueError> {
        Ok(Box::new(T::try_from_value(value)?))
    }
}

impl<T> DatexProxyTypes for Box<T>
where
    T: DatexProxyTypes,
{
    fn datex_type(memory: &mut SharedReferencesCache) -> Type {
        T::datex_type(memory)
    }
}
