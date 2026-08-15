use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    types::{
        entities::entity_type_definition::EntityTypeDefinition, r#type::Type,
    },
    values::{
        core_values::{
            callable::Callable, endpoint::Endpoint, list::List, map::Map,
            range::Range,
        },
        value::Value,
    },
};

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::TypeDefinition,
};

macro_rules! impl_datex_direct_via_value_container {
    ($type:ty, $dx_type:expr) => {
        impl DatexValueProxy for $type {}

        impl DatexValueProxyInfallibleSerialize for $type {
            fn to_value(self) -> Value {
               Value::from(self)
            }
        }
        impl DatexValueProxySerialize for $type {
            fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
                Ok(Value::from(self))
            }
        }
        impl DatexValueProxyDeserialize for $type {
            fn try_from_value(
                value: Value,
            ) -> Result<Self, TryFromDatexValueError> {
               value.try_into().map_err(|_| TryFromDatexValueError(format!("Cannot cast ValueContainer to {}, expected ValueContainer::Local with inner type {}", stringify!($type), stringify!($type))))
            }
        }

        impl DatexProxyTypes for $type {
            fn datex_type(_memory: &mut SharedReferencesCache) -> Type {
                Type::Definition(TypeDefinition::CoreType($dx_type.into()).into())
            }
        }
    };
}

impl_datex_direct_via_value_container!(Endpoint, CoreLibBaseTypeId::Endpoint);
impl_datex_direct_via_value_container!(Map, CoreLibBaseTypeId::Map);
impl_datex_direct_via_value_container!(List, CoreLibBaseTypeId::List);
impl_datex_direct_via_value_container!(Range, CoreLibBaseTypeId::Range);
impl_datex_direct_via_value_container!(Type, CoreLibBaseTypeId::Type);
impl_datex_direct_via_value_container!(
    EntityTypeDefinition,
    CoreLibBaseTypeId::Any
);
impl_datex_direct_via_value_container!(Callable, CoreLibBaseTypeId::Callable);
