//! Implements [DatexValueProxy] for [CoreValue](crate::values::core_values) implementation. That allows to convert e.g. [Endpoint] to [Value] and back.
//! Also implements [DatexProxyType] to provide the correct [Type] for each implementation.

use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    types::{
        entities::entity_type_definition::EntityTypeDefinition, r#type::Type,
    },
    values::{
        core_values::{
            boolean::Boolean, callable::Callable, decimal::Decimal,
            endpoint::Endpoint, integer::Integer, list::List, map::Map,
            native::DatexNative, range::Range, text::Text,
        },
        value::Value,
    },
};
use core::any::Any;

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::TypeDefinition,
};

/// Implements [DatexValueProxy] for a [CoreValue](crate::values::core_values) implementation.
/// This allows to convert e.g. [Endpoint] to [ValueContainer] and back.
/// Also implements [DatexProxyType] to provide the correct [Type] for each implementation.
/// The `gen` param defines, for which concrete context to impl the serialization traits.
macro_rules! impl_datex_direct_via_value_container {
    ($type:ty, $dx_type:expr) => {
        impl DatexValueProxy for $type {}

        impl DatexValueProxyInfallibleSerialize for $type {
            fn boxed_to_value(self: Box<Self>, _context: &mut SharedReferencesCache) -> Value {
               Value::from(*self)
            }
        }
        impl DatexValueProxySerialize for $type {
            fn try_boxed_to_value(self: Box<Self>, _context: &mut SharedReferencesCache) -> Result<Value, TryToDatexValueError> {
                Ok(Value::from(*self))
            }
        }
        impl DatexValueProxyDeserialize for $type {
            fn try_from_value(
                value: Value,
            ) -> Result<Self, TryFromDatexValueError> {
               value.try_into().map_err(|_| TryFromDatexValueError(format!("Cannot cast ValueContainer to {}, expected ValueContainer::Local with inner type {}", stringify!($type), stringify!($type))))
            }
        }

        impl DatexNative for $type {
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

        impl DatexProxyType for $type {
            fn datex_type(_context: &mut SharedReferencesCache) -> Type {
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
impl_datex_direct_via_value_container!(Integer, CoreLibBaseTypeId::Integer);
impl_datex_direct_via_value_container!(Decimal, CoreLibBaseTypeId::Decimal);
impl_datex_direct_via_value_container!(Text, CoreLibBaseTypeId::Text);
impl_datex_direct_via_value_container!(Boolean, CoreLibBaseTypeId::Boolean);
// impl_datex_direct_via_value_container!(Instant, CoreLibBaseTypeId::Instant);

#[cfg(test)]
mod tests {
    use crate::{
        datex_proxy::{
            DatexValueProxyDeserialize, DatexValueProxyInfallibleSerialize,
        },
        values::{
            core_value::CoreValue, core_values::endpoint::Endpoint,
            value::Value,
        },
    };

    #[test]
    fn to_value() {
        let endpoint = Endpoint::new("@jonas");
        let value: Value = endpoint.clone().to_value_without_cache();
        assert!(matches!(
            value.inner,
            CoreValue::Endpoint(ref e) if e == &endpoint
        ));
    }

    #[test]
    fn try_boxed_to_value() {
        let endpoint = Endpoint::new("@jonas");
        let value: Value = endpoint.clone().to_value_without_cache();
        assert!(matches!(
            value.inner,
            CoreValue::Endpoint(ref e) if e == &endpoint
        ));
    }

    #[test]
    fn try_from_value() {
        let endpoint = Endpoint::new("@jonas");
        let value: Value = endpoint.clone().to_value_without_cache();
        let result: Result<Endpoint, _> = Endpoint::try_from_value(value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), endpoint);
    }
}
