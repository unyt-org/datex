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

use crate::dif::serde_context::SerdeContext;
use core::fmt;
use core::str::FromStr;
use serde::{
    Deserializer,
    de::{DeserializeSeed, SeqAccess, Visitor},
};
use crate::values::core_values::endpoint::Endpoint;

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
                Ok(CoreValue::Integer(Integer::try_from_string(v).map_err(|e| {
                    E::custom(format!("failed to parse integer: {e}"))
                })?))
            }
            CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal) => {
                Ok(CoreValue::Decimal(Decimal::try_from_string(v).map_err(|e| {
                    E::custom(format!("failed to parse decimal: {e}"))
                })?))
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
            )) => Ok(CoreValue::TypedDecimal(TypedDecimal::F64((v as f64).into()))),
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
           )) => Ok(CoreValue::TypedDecimal(TypedDecimal::F32((v as f32).into()))),
            CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
               DecimalTypeVariant::DBig,
           )) => Ok(CoreValue::TypedDecimal(TypedDecimal::Decimal(
                v.into(),
            ))),
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
// FIXME -> we can remove this?!
// / Deserialization for [CoreValue] using a [DeserializationContext] to provide access to the memory during deserialization.
// impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, CoreValue> {
//     type Value = CoreValue;

//     fn deserialize<D: Deserializer<'de>>(
//         self,
//         deserializer: D,
//     ) -> Result<CoreValue, D::Error> {
//         deserializer.deserialize_any(self)
//     }
// }

// impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, CoreValue> {
//     type Value = CoreValue;

//     fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.write_str("a CoreValue")
//     }

//     fn visit_bool<E>(self, value: bool) -> Result<CoreValue, E> {
//         Ok(CoreValue::Boolean(value.into()))
//     }

//     fn visit_i64<E>(self, value: i64) -> Result<CoreValue, E> {
//         Ok(CoreValue::Integer(value.into()))
//     }

//     fn visit_u64<E>(self, value: u64) -> Result<CoreValue, E> {
//         Ok(CoreValue::Integer(value.into()))
//     }

//     fn visit_f64<E>(self, value: f64) -> Result<CoreValue, E> {
//         Ok(CoreValue::Decimal(value.into()))
//     }

//     fn visit_str<E>(self, value: &str) -> Result<CoreValue, E>
//     where
//         E: serde::de::Error,
//     {
//         Ok(CoreValue::Text(value.into()))
//     }

//     fn visit_string<E>(self, value: String) -> Result<CoreValue, E> {
//         Ok(CoreValue::Text(value.into()))
//     }

//     fn visit_seq<A: SeqAccess<'de>>(
//         mut self,
//         mut seq: A,
//     ) -> Result<CoreValue, A::Error> {
//         let mut items = Vec::new();
//         while let Some(item) =
//             seq.next_element_seed(self.cast::<ValueContainer>())?
//         {
//             items.push(item);
//         }
//         Ok(CoreValue::List(List::from(items)))
//     }

//     // fn visit_map<A>(mut self, mut map: A) -> Result<CoreValue, A::Error>
//     // where
//     //     A: MapAccess<'de>,
//     // {
//     //     let mut items = Vec::new();

//     //     while let Some(key) = {
//     //         let key_seed = self.cast::<ValueContainer>();
//     //         map.next_key_seed(key_seed)?
//     //     } {
//     //         let value = {
//     //             let value_seed = self.cast::<ValueContainer>();
//     //             map.next_value_seed(value_seed)?
//     //         };

//     //         items.push((key, value));
//     //     }

//     //     Ok(CoreValue::Map(Map::from(items)))
//     // }
//     fn visit_map<A>(mut self, mut map: A) -> Result<CoreValue, A::Error>
//     where
//         A: MapAccess<'de>,
//     {
//         use serde::de::Error;

//         let mut ty: Option<String> = None;
//         let mut out: Option<CoreValue> = None;

//         while let Some(field) = map.next_key::<String>()? {
//             match field.as_str() {
//                 "$type" => {
//                     ty = Some(map.next_value()?);
//                 }

//                 "value" => {
//                     let ty = ty.as_deref().ok_or_else(|| {
//                         A::Error::custom("`$type` must come before `value`")
//                     })?;

//                     let value = match ty {
//                         "map" => {
//                             let value =
//                                 map.next_value_seed(self.cast::<Map>())?;
//                             CoreValue::Map(value)
//                         }

//                         "range" => {
//                             let value =
//                                 map.next_value_seed(self.cast::<Range>())?;
//                             CoreValue::Range(value)
//                         }

//                         "endpoint" => {
//                             let value = map.next_value()?;
//                             CoreValue::Endpoint(value)
//                         }

//                         other => {
//                             return Err(A::Error::custom(format!(
//                                 "unknown CoreValue `$type`: {other}"
//                             )));
//                         }
//                     };

//                     out = Some(value);
//                 }

//                 other => {
//                     return Err(A::Error::custom(format!(
//                         "unexpected field `{other}` in CoreValue object"
//                     )));
//                 }
//             }
//         }

//         out.ok_or_else(|| {
//             A::Error::custom("missing `value` field in CoreValue object")
//         })
//     }
// }
