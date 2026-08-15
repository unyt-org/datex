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

macro_rules! impl_try_from_core_value {
    ($($variant:ident => $type:ty),* $(,)?) => {
        $(
            impl TryFrom<Value> for $type {
                type Error = TryFromDatexValueError;
                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    value.inner.try_into()
                }
            }

            impl TryFrom<CoreValue> for $type {
                type Error = TryFromDatexValueError;
                fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
                    match value {
                        CoreValue::$variant(v) => Ok(v),
                        _ => Err(TryFromDatexValueError(format!("Cannot cast CoreValue to {}, expected CoreValue::{}", stringify!($type), stringify!($variant)))),
                    }
                }
            }

            impl<'a> TryFrom<&'a CoreValue> for &'a $type {
                type Error = TryFromDatexValueError;
                fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
                    match value {
                        CoreValue::$variant(v) => Ok(v),
                        _ => Err(TryFromDatexValueError(format!("Cannot cast CoreValue to {}, expected CoreValue::{}", stringify!($type), stringify!($variant)))),
                    }
                }
            }

            impl<'a> TryFrom<&'a mut CoreValue> for &'a mut $type {
                type Error = TryFromDatexValueError;
                fn try_from(value: &'a mut CoreValue) -> Result<Self, Self::Error> {
                    match value {
                        CoreValue::$variant(v) => Ok(v),
                        _ => Err(TryFromDatexValueError(format!("Cannot cast CoreValue to {}, expected CoreValue::{}", stringify!($type), stringify!($variant)))),
                    }
                }
            }
        )*
    };
}

// Implement [TryFrom] for each CoreValue variant
impl_try_from_core_value! {
    Integer             => Integer,
    TypedInteger        => TypedInteger,
    Decimal             => Decimal,
    TypedDecimal        => TypedDecimal,
    Boolean             => Boolean,
    Endpoint            => Endpoint,
    Text                => Text,
    List                => List,
    Map                 => Map,
    Type                => Type,
    EntityTypeDefinition => EntityTypeDefinition,
    Range               => Range,
    Callable            => Callable,
}
