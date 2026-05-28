use crate::{
    libs::core::type_id::{
        CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId,
    },
    prelude::*,
    values::{
        core_value::CoreValue,
        core_values::{
            decimal::{
                Decimal,
                typed_decimal::{DecimalTypeVariant, TypedDecimal},
            },
            integer::{
                Integer,
                typed_integer::{IntegerTypeVariant, TypedInteger},
            },
            list::List,
            map::Map,
            range::Range,
        },
        value_container::ValueContainer,
    },
};
use serde::de::{Error, MapAccess};

use crate::{
    dif::serde_context::SerdeContext, values::core_values::endpoint::Endpoint,
};
use core::{fmt, str::FromStr};
use serde::{
    Deserializer,
    de::{DeserializeSeed, SeqAccess, Visitor},
};

pub struct CoreValueVisitor {
    pub core_lib_id: CoreLibTypeId,
}

impl<'de, 'ctx> DeserializeSeed<'de> for CoreValueVisitor {
    type Value = CoreValue;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<CoreValue, D::Error> {
        deserializer.deserialize_any(self)
    }
}
impl<'de, 'ctx> Visitor<'de> for CoreValueVisitor {
    type Value = CoreValue;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a CoreValue")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match self.core_lib_id {
            CoreLibTypeId::Base(CoreLibBaseTypeId::Null) => Ok(CoreValue::Null),
            other => Err(E::custom(format!(
                "expected CoreValue of type Unit, got {other}"
            ))),
        }
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_unit()
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match self.core_lib_id {
            CoreLibTypeId::Base(CoreLibBaseTypeId::Boolean) => {
                Ok(CoreValue::Boolean(v.into()))
            }
            other => Err(E::custom(format!(
                "expected CoreValue of type Boolean, got {other}"
            ))),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.core_lib_id {
            CoreLibTypeId::Base(CoreLibBaseTypeId::Text) => {
                Ok(CoreValue::Text(v.into()))
            }
            CoreLibTypeId::Base(CoreLibBaseTypeId::Endpoint) => {
                Ok(CoreValue::Endpoint(Endpoint::from_str(v).map_err(|e| {
                    E::custom(format!("failed to parse endpoint: {e}"))
                })?))
            }
            CoreLibTypeId::Base(CoreLibBaseTypeId::Integer) => {
                Ok(CoreValue::Integer(Integer::try_from_string(v).map_err(
                    |e| E::custom(format!("failed to parse integer: {e}")),
                )?))
            }
            CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal) => {
                Ok(CoreValue::Decimal(Decimal::try_from_string(v).map_err(
                    |e| E::custom(format!("failed to parse decimal: {e}")),
                )?))
            }
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::IBig,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::IBig(
                Integer::try_from_string(v).map_err(|e| {
                    E::custom(format!("failed to parse integer: {e}"))
                })?,
            ))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::DBig,
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::Decimal(
                Decimal::try_from_string(v).map_err(|e| {
                    E::custom(format!("failed to parse decimal: {e}"))
                })?,
            ))),
            other => Err(E::custom(format!(
                "expected CoreValue of type Text, got {other}"
            ))),
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_i128(v as i128)
    }
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: Error,
    {
        todo!()
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match self.core_lib_id {
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::F32,
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::F32(v.into()))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::F64,
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::F64(
                (v as f64).into(),
            ))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::DBig,
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::Decimal(
                (v as f64).into(),
            ))),
            CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal) => {
                Ok(CoreValue::Decimal(Decimal::from(v)))
            }
            other => Err(E::custom(format!(
                "expected CoreValue of type Decimal, got {other}"
            ))),
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match self.core_lib_id {
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::F64,
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::F64(v.into()))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::F32,
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::F32(
                (v as f32).into(),
            ))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::DBig,
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::Decimal(v.into()))),
            CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal) => {
                Ok(CoreValue::Decimal(Decimal::from(v)))
            }
            other => Err(E::custom(format!(
                "expected CoreValue of type Decimal, got {other}"
            ))),
        }
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match self.core_lib_id {
            // integer
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::I8,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::I8(v as i8))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::I16,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::I16(v as i16))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::I32,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::I32(v as i32))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::I64,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::I64(v as i64))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::I128,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::I128(v))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::IBig,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::IBig(v.into()))),

            // unsigned integer
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::U8,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::U8(v as u8))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::U16,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::U16(v as u16))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::U32,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::U32(v as u32))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::U64,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::U64(v as u64))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                IntegerTypeVariant::U128,
            )) => Ok(CoreValue::TypedInteger(TypedInteger::U128(v as u128))),

            // base
            CoreLibTypeId::Base(CoreLibBaseTypeId::Integer) => {
                Ok(CoreValue::Integer(v.into()))
            }
            CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal) => {
                Ok(CoreValue::Decimal(
                    Decimal::try_from_string(&v.to_string()).map_err(|e| {
                        E::custom(format!("failed to parse decimal: {e}"))
                    })?,
                ))
            }

            // decimal
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::F32,
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::F32(
                (v as f32).into(),
            ))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::F64,
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::F64(
                (v as f64).into(),
            ))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                DecimalTypeVariant::DBig,
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::Decimal(
                (v as f64).into(),
            ))),

            other => Err(E::custom(format!(
                "expected CoreValue of type Integer, got {other}"
            ))),
        }
    }
}
