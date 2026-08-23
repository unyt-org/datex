//! Implements [TryFrom] and [TryInto] for Rust native types to and from DATEX [CoreValue], [Value] and [ValueContainer] types.
//! This allows to convert [u8] into DATEX [Value] and [ValueContainer] and allows to convert [CoreValue], [Value] and [ValueContainer] into [u8].
mod bool;
mod integers;
mod floats;
mod string;
mod hash_map;
mod option;
mod r#box;
mod vec;
mod duration;

use core::any::Any;
use num::ToPrimitive;

use crate::{
    datex_proxy::{TryFromDatexValueError, TryToDatexValueError, *},
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibVariantTypeId},
    prelude::*,
    types::r#type::Type,
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean, decimal::typed_decimal::TypedDecimal,
            integer::typed_integer::TypedInteger, native::DatexNative,
            text::Text,
        },
        value::Value,
        value_container::ValueContainer,
    },
};

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    types::type_definition::TypeDefinition,
    values::core_values::{
        decimal::typed_decimal::DecimalTypeVariant,
        integer::typed_integer::IntegerTypeVariant,
    },
};
use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut};

/// Implements [TryFrom] and [TryInto] for Rust core types to and from DATEX [CoreValue], [Value] and [ValueContainer] types.
/// Also implements [DatexValueProxy] for Rust core types to provide the correct [Type] for each implementation.
macro_rules! implement_rust_native_traits {
    ($type:ty, $dx_type:expr, {$($core_match:tt)*}) => {
        impl TryFrom<CoreValue> for $type {
            type Error = TryFromDatexValueError;

            fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
               match value {
                    $($core_match)*
                    _ => Err(TryFromDatexValueError(format!("Cannot cast CoreValue to {}", stringify!($type)))),
               }
            }
        }

        impl TryFrom<Value> for $type {
            type Error = TryFromDatexValueError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                value.inner.try_into()
            }
        }

        impl TryFrom<ValueContainer> for $type {
            type Error = TryFromDatexValueError;

            fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
                match value {
                    ValueContainer::Local(value) => value.try_into(),
                    _ => Err(TryFromDatexValueError(format!("Cannot cast ValueContainer to {}, expected ValueContainer::Local", stringify!($type)))),
                }
            }
        }

        // specialized unit impl:
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

        // deserialize
        impl DatexValueProxyDeserialize for $type {
            fn try_from_value(
                value: Value,
            ) -> Result<Self, TryFromDatexValueError> {
                value.try_into()
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
implement_rust_native_traits!(
    bool,
    CoreLibBaseTypeId::Boolean,
    {
        CoreValue::Boolean(Boolean(value)) => Ok(value),
    }
);

implement_rust_native_traits!(
    u8,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U8),
    {
        CoreValue::TypedInteger(TypedInteger::U8(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_u8().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u8, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u8().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u8, value is not an integer", value))),
    }
);
implement_rust_native_traits!(
    u16,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U16),
    {
        CoreValue::TypedInteger(TypedInteger::U16(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_u16().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u16, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u16().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u16, value is not an integer", value))),
    }
);
implement_rust_native_traits!(
    u32,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U32),
    {
        CoreValue::TypedInteger(TypedInteger::U32(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_u32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u32, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u32, value is not an integer", value))),
    }
);
implement_rust_native_traits!(
    u64,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U64),
    {
        CoreValue::TypedInteger(TypedInteger::U64(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_u64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u64, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to u64, value is not an integer", value))),
    }
);

// usize depending on platform
#[cfg(target_pointer_width = "32")]
implement_rust_native_traits!(
    usize,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U32),
    {
        CoreValue::TypedInteger(TypedInteger::U32(value)) => Ok(value as usize),
        CoreValue::TypedInteger(value) => value.to_u32().map(|v| v as usize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to usize, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u32().map(|v| v as usize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to usize, value is not an integer", value))),
    }
);
#[cfg(target_pointer_width = "64")]
implement_rust_native_traits!(
    usize,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U64),
    {
        CoreValue::TypedInteger(TypedInteger::U64(value)) => Ok(value as usize),
        CoreValue::TypedInteger(value) => value.to_u64().map(|v| v as usize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to usize, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_u64().map(|v| v as usize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to usize, value is not an integer", value))),
    }
);

implement_rust_native_traits!(
    i8,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I8),
    {
        CoreValue::TypedInteger(TypedInteger::I8(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_i8().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i8, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i8().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i8, value is not an integer", value))),
    }
);
implement_rust_native_traits!(
    i16,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I16),
    {
        CoreValue::TypedInteger(TypedInteger::I16(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_i16().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i16, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i16().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i16, value is not an integer", value))),
    }
);
implement_rust_native_traits!(
    i32,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I32),
    {
        CoreValue::TypedInteger(TypedInteger::I32(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_i32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i32, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i32, value is not an integer", value))),
    }
);
implement_rust_native_traits!(
    i64,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I64),
    {
        CoreValue::TypedInteger(TypedInteger::I64(value)) => Ok(value),
        CoreValue::TypedInteger(value) => value.to_i64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i64, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to i64, value is not an integer", value))),
    }
);

// isize depending on platform
#[cfg(target_pointer_width = "32")]
implement_rust_native_traits!(
    isize,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I32),
    {
        CoreValue::TypedInteger(TypedInteger::I32(value)) => Ok(value as isize),
        CoreValue::TypedInteger(value) => value.to_i32().map(|v| v as isize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to isize, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i32().map(|v| v as isize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to isize, value is not an integer", value))),
    }
);
#[cfg(target_pointer_width = "64")]
implement_rust_native_traits!(
    isize,
    CoreLibVariantTypeId::Integer(IntegerTypeVariant::I64),
    {
        CoreValue::TypedInteger(TypedInteger::I64(value)) => Ok(value as isize),
        CoreValue::TypedInteger(value) => value.to_i64().map(|v| v as isize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to isize, value is not an integer", value))),
        CoreValue::TypedDecimal(value) => value.to_i64().map(|v| v as isize).ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to isize, value is not an integer", value))),
    }
);

implement_rust_native_traits!(
    f32,
    CoreLibVariantTypeId::Decimal(DecimalTypeVariant::F32),
    {
        CoreValue::TypedDecimal(TypedDecimal::F32(value)) => Ok(value.into()),
        CoreValue::TypedInteger(value) => value.to_f32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to f32, value is not a decimal", value))),
        CoreValue::TypedDecimal(value) => value.to_f32().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to f32, value is not a decimal", value))),
    }
);
implement_rust_native_traits!(
    f64,
    CoreLibVariantTypeId::Decimal(DecimalTypeVariant::F64),
    {
        CoreValue::TypedDecimal(TypedDecimal::F64(value)) => Ok(value.into()),
        CoreValue::TypedInteger(value) => value.to_f64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to f64, value is not a decimal", value))),
        CoreValue::TypedDecimal(value) => value.to_f64().ok_or_else(|| TryFromDatexValueError(format!("Cannot cast {} to f64, value is not a decimal", value))),
    }
);
implement_rust_native_traits!(
    String,
    CoreLibBaseTypeId::Text,
    {
        CoreValue::Text(Text(value)) => Ok(value),
    }
);

// &str
impl<'a> TryFrom<&'a CoreValue> for &'a str {
    type Error = TryFromDatexValueError;

    fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Text(Text(value)) => Ok(value.as_str()),
            _ => Err(TryFromDatexValueError(
                "Cannot cast CoreValue to &str".into(),
            )),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = TryFromDatexValueError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        (&value.inner).try_into()
    }
}
impl<'a> TryFrom<&'a ValueContainer> for &'a str {
    type Error = TryFromDatexValueError;

    fn try_from(value: &'a ValueContainer) -> Result<Self, Self::Error> {
        match value {
            ValueContainer::Local(value) => value.try_into(),
            _ => Err(TryFromDatexValueError(
                "Cannot cast shared ValueContainer to &str".into(),
            )),
        }
    }
}
impl DatexProxyType for &str {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Text.into()).into(),
        )
    }
}

impl DatexProxyType for str {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::Definition(
            TypeDefinition::CoreType(CoreLibBaseTypeId::Text.into()).into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        datex_proxy::{
            DatexValueProxyInfallibleSerialize, DatexValueProxySerialize,
            TryFromDatexValueError, TryToDatexValueError,
        },
        values::{
            core_value::CoreValue,
            core_values::{boolean::Boolean, text::Text},
            value::Value,
        },
    };

    #[test]
    fn try_without_context() {
        // these rust types should have the to_value_container_without_cache
        "test".to_string().to_value_container_without_cache();
    }

    #[test]
    fn try_from_core_value() {
        let value = CoreValue::Text(Text("Hello, World!".to_string()));
        let result: Result<String, TryFromDatexValueError> = value.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn try_from_value() {
        let value =
            Value::from(CoreValue::Text(Text("Hello, World!".to_string())));
        let result: Result<String, TryFromDatexValueError> = value.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn to_value() {
        let value = true;
        let result: Value = value.to_value_without_cache();
        assert_eq!(result, Value::from(CoreValue::Boolean(Boolean(true))));
    }

    #[test]
    fn try_boxed_to_value() {
        let value = Box::new(true);
        let result: Result<Value, TryToDatexValueError> =
            value.try_boxed_to_value(&mut SharedReferencesCache::default());
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::from(CoreValue::Boolean(Boolean(true)))
        );
    }
}
