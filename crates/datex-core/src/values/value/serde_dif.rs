use core::fmt;

use crate::{
    dif::serde_context::SerdeContext,
    libs::core::{
        core_lib_id::{CoreLibId, CoreLibIdIndex},
        type_id::{CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId},
    },
    prelude::*,
    types::type_definition::TypeDefinition,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean, decimal::typed_decimal::TypedDecimal,
            endpoint::Endpoint, integer::typed_integer::TypedInteger,
            list::List, map::Map, range::Range,
        },
        value::Value,
    },
};
use num::ToPrimitive;
use ordered_float::OrderedFloat;
use serde::{
    Deserialize, Serialize, Serializer,
    de::Visitor,
    ser::{SerializeStruct, SerializeTuple},
};

impl<'ctx> SerdeContext<'ctx, Value> {
    /// This method is used to serialize a value that can be represented directly (e.g. a boolean or a text)
    /// or with a custom type definition (e.g. a nominal type).
    /// ## For no custom type:
    ///   true -> "true"
    ///   "Hello" -> "Hello"
    ///   42f32 -> 42
    /// ## For custom type:
    /// {custom_type: LiteralTypeDefinition::Integer(42), value: 42} -> [<type_definition>, 42]
    fn serialize_direct<Se, T>(
        &mut self,
        inner: &T,
        custom_type: &Option<TypeDefinition>,
        serializer: Se,
    ) -> Result<Se::Ok, Se::Error>
    where
        T: Serialize + ?Sized,
        Se: Serializer,
    {
        match custom_type {
            Some(custom_type) => {
                // [null, custom_type, value]
                let mut tuple = serializer.serialize_tuple(3)?;
                tuple.serialize_element(&())?;
                tuple.serialize_element(&ValueWithSeed::new(
                    custom_type,
                    &mut self.cast::<TypeDefinition>(),
                ))?;
                tuple.serialize_element(inner)?;
                tuple.end()
            }
            // <direct>
            None => inner.serialize(serializer),
        }
    }
    fn serialize_direct_serde<Se, T>(
        &mut self,
        inner: &T,
        custom_type: &Option<TypeDefinition>,
        serializer: Se,
    ) -> Result<Se::Ok, Se::Error>
    where
        T: Sized,
        Se: Serializer,
        for<'a> SerdeContext<'a, T>: SerializeSeed<Value = T>,
    {
        match custom_type {
            Some(custom_type) => {
                // [null, custom_type, value]
                let mut tuple = serializer.serialize_tuple(3)?;
                tuple.serialize_element(&())?;
                tuple.serialize_element(&ValueWithSeed::new(
                    custom_type,
                    &mut self.cast::<TypeDefinition>(),
                ))?;
                tuple.serialize_element(&ValueWithSeed::new(
                    inner,
                    &mut self.cast::<T>(),
                ))?;
                tuple.end()
            }
            // <direct>
            None => ValueWithSeed::new(inner, &mut self.cast::<T>())
                .serialize(serializer),
        }
    }

    /// This method is used to serialize a value that can be represented directly (e.g. an endpoint)
    /// Depending on the custom type, this will either serialize
    fn serialize_with_core_type<Se, T>(
        &mut self,
        inner: &T,
        core_lib_id: CoreLibId,
        custom_type: &Option<TypeDefinition>,
        serializer: Se,
    ) -> Result<Se::Ok, Se::Error>
    where
        T: Serialize + ?Sized,
        Se: Serializer,
    {
        let index = CoreLibIdIndex::from(core_lib_id);
        let mut tuple = serializer.serialize_tuple(3)?;
        tuple.serialize_element(&index.to_u16())?;
        match custom_type {
            Some(custom_type) => {
                // [id, custom_type, value]
                tuple.serialize_element(&ValueWithSeed::new(
                    custom_type,
                    &mut self.cast::<TypeDefinition>(),
                ))?;
                tuple.serialize_element(inner)?;
            }
            None => {
                // [id, null, value]
                tuple.serialize_element(&())?;
                tuple.serialize_element(inner)?;
            }
        }
        tuple.end()
    }

    fn serialize_with_core_type_serde<Se, T>(
        &mut self,
        inner: &T,
        core_lib_id: CoreLibId,
        custom_type: &Option<TypeDefinition>,
        serializer: Se,
    ) -> Result<Se::Ok, Se::Error>
    where
        T: Sized,
        Se: Serializer,
        for<'a> SerdeContext<'a, T>: SerializeSeed<Value = T>,
    {
        let index = CoreLibIdIndex::from(core_lib_id);
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&index.to_u16())?;

        match custom_type {
            Some(custom_type) => {
                // [id, custom_type, value]
                tuple.serialize_element(&ValueWithSeed::new(
                    custom_type,
                    &mut self.cast::<TypeDefinition>(),
                ))?;
                tuple.serialize_element(&ValueWithSeed::new(
                    inner,
                    &mut self.cast::<T>(),
                ))?;
            }
            None => {
                // [id, null, value]
                tuple.serialize_element(&())?;
                tuple.serialize_element(&ValueWithSeed::new(
                    inner,
                    &mut self.cast::<T>(),
                ))?;
            }
        }
        tuple.end()
    }
}

/// Serialization for [Value].
impl<'ctx> SerializeSeed for SerdeContext<'ctx, Value> {
    type Value = Value;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &value.inner {
            // Direct serializable core values, that can be serialized as they can be unambiguously deserialized without it
            CoreValue::Boolean(b) => {
                self.serialize_direct(b, &value.custom_type, serializer)
            }
            CoreValue::Text(s) => {
                self.serialize_direct(s, &value.custom_type, serializer)
            }
            CoreValue::Null => {
                self.serialize_direct(&(), &value.custom_type, serializer)
            }
            CoreValue::TypedDecimal(TypedDecimal::F32(OrderedFloat(f32))) => {
                self.serialize_direct(f32, &value.custom_type, serializer)
            }
            CoreValue::Map(Map::StructuralWithStringKeys(map)) => {
                self.serialize_direct(map, &value.custom_type, serializer)
            }

            // Core values that require a specific core type id to be serialized for non-ambiguous deserialization
            CoreValue::Endpoint(endpoint) => self.serialize_with_core_type(
                endpoint,
                CoreLibTypeId::Base(CoreLibBaseTypeId::Endpoint).into(),
                &value.custom_type,
                serializer,
            ),
            CoreValue::Decimal(d) => self.serialize_with_core_type(
                d,
                CoreLibTypeId::Base(CoreLibBaseTypeId::Decimal).into(),
                &value.custom_type,
                serializer,
            ),
            CoreValue::Integer(i) => self.serialize_with_core_type(
                i,
                CoreLibTypeId::Base(CoreLibBaseTypeId::Integer).into(),
                &value.custom_type,
                serializer,
            ),
            CoreValue::TypedInteger(ti) => self.serialize_with_core_type(
                ti,
                CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(
                    ti.variant(),
                ))
                .into(),
                &value.custom_type,
                serializer,
            ),
            CoreValue::TypedDecimal(td) => self.serialize_with_core_type(
                td,
                CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(
                    td.variant(),
                ))
                .into(),
                &value.custom_type,
                serializer,
            ),

            // Complex core values, that can contain nested values
            CoreValue::List(l) => self.serialize_with_core_type_serde(
                l,
                CoreLibTypeId::Base(CoreLibBaseTypeId::List).into(),
                &value.custom_type,
                serializer,
            ),
            CoreValue::Range(range) => self.serialize_with_core_type_serde(
                range,
                CoreLibTypeId::Base(CoreLibBaseTypeId::Range).into(),
                &value.custom_type,
                serializer,
            ),

            CoreValue::Map(map) => self.serialize_with_core_type_serde(
                map,
                CoreLibTypeId::Base(CoreLibBaseTypeId::Map).into(),
                &value.custom_type,
                serializer,
            ),
            CoreValue::Type(ty) => todo!(),
            CoreValue::NominalTypeDefinition(nominal_type_definition) => {
                todo!()
            }
            CoreValue::Callable(callable) => todo!(),
        }
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Value> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("struct Value with a 'value' property")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value {
            inner: CoreValue::Boolean(Boolean::new(v)),
            custom_type: None,
        })
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value {
            inner: CoreValue::Text(v.into()),
            custom_type: None,
        })
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value {
            inner: CoreValue::Null,
            custom_type: None,
        })
    }
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value {
            inner: CoreValue::TypedDecimal(TypedDecimal::F32(OrderedFloat(v))),
            custom_type: None,
        })
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f32(v as f32)
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f32(v as f32)
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_unit()
    }
    fn visit_map<A>(self, _map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        unimplemented!("deserialization of maps is not implemented yet")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let first: Option<u16> = seq.next_element()?;

        let core_lib_id = first
            .map(CoreLibIdIndex::new)
            .map(CoreLibTypeId::try_from)
            .transpose()
            .map_err(|_| {
                serde::de::Error::custom(format!(
                    "invalid core lib id index: {:?}",
                    first
                ))
            })?;

        let custom_type: Option<TypeDefinition> =
            seq.next_element_seed(self.cast::<TypeDefinition>())?;

        let inner: CoreValue =
            self.next_core_value_by_type_id(&mut seq, core_lib_id)?;
        Ok(Value { custom_type, inner })
    }
}

impl<'ctx, T> SerdeContext<'ctx, T> {
    fn next_core_value_by_type_id<'de, A>(
        &mut self,
        seq: &mut A,
        core_lib_id: Option<CoreLibTypeId>,
    ) -> Result<CoreValue, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let Some(core_lib_id) = core_lib_id else {
            return seq.next_element()?.ok_or_else(|| {
                serde::de::Error::custom(
                    "expected a core value with a core lib id",
                )
            });
        };
        match core_lib_id {
            CoreLibTypeId::Base(base) => self.next_base_core_value(seq, base),
            CoreLibTypeId::Variant(variant) => {
                self.next_variant_core_value(seq, variant)
            }
        }
    }

    fn next_base_core_value<'de, A>(
        &mut self,
        seq: &mut A,
        base: CoreLibBaseTypeId,
    ) -> Result<CoreValue, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        match base {
            CoreLibBaseTypeId::Endpoint => {
                Ok(CoreValue::Endpoint(self.next_required(seq)?))
            }
            CoreLibBaseTypeId::Decimal => {
                Ok(CoreValue::Decimal(self.next_required(seq)?))
            }
            CoreLibBaseTypeId::Integer => {
                Ok(CoreValue::Integer(self.next_required(seq)?))
            }
            CoreLibBaseTypeId::List => {
                let list =
                    seq.next_element_seed(self.cast::<List>())?.ok_or_else(
                        || serde::de::Error::invalid_length(2, &"list"),
                    )?;

                Ok(CoreValue::List(list))
            }
            CoreLibBaseTypeId::Range => {
                let range =
                    seq.next_element_seed(self.cast::<Range>())?.ok_or_else(
                        || serde::de::Error::invalid_length(2, &"range"),
                    )?;
                Ok(CoreValue::Range(range))
            }
            CoreLibBaseTypeId::Map => {
                let map =
                    seq.next_element_seed(self.cast::<Map>())?.ok_or_else(
                        || serde::de::Error::invalid_length(2, &"map"),
                    )?;

                Ok(CoreValue::Map(map))
            }
            CoreLibBaseTypeId::Type => {
                unimplemented!(
                    "deserialization of type values is not implemented yet"
                )
            }
            CoreLibBaseTypeId::Null => Ok(CoreValue::Null),
            CoreLibBaseTypeId::Text => {
                let text: String = self.next_required(seq)?;
                Ok(CoreValue::Text(text.into()))
            }
            CoreLibBaseTypeId::Boolean => {
                let boolean: bool = self.next_required(seq)?;
                Ok(CoreValue::Boolean(Boolean::new(boolean)))
            }
            CoreLibBaseTypeId::Unit => Ok(CoreValue::Null),
            CoreLibBaseTypeId::Never => unimplemented!(),
            CoreLibBaseTypeId::Unknown => todo!(),
            CoreLibBaseTypeId::Callable => todo!(),
        }
    }

    fn next_variant_core_value<'de, A>(
        &mut self,
        seq: &mut A,
        variant: CoreLibVariantTypeId,
    ) -> Result<CoreValue, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        match variant {
            CoreLibVariantTypeId::Integer(variant) => {
                let value: String = self.next_required(seq)?;

                Ok(CoreValue::TypedInteger(
                    TypedInteger::from_string_and_variant(&value, variant)
                        .map_err(|err| {
                            serde::de::Error::custom(format!(
                                "invalid typed integer value `{value}` for variant `{variant}`: {err}"
                            ))
                        })?,
                ))
            }

            CoreLibVariantTypeId::Decimal(variant) => {
                let value: String = self.next_required(seq)?;

                Ok(CoreValue::TypedDecimal(
                    TypedDecimal::from_string_and_variant(&value, variant)
                        .map_err(|err| {
                            serde::de::Error::custom(format!(
                                "invalid typed decimal value `{value}` for variant `{variant}`: {err}"
                            ))
                        })?,
                ))
            }
        }
    }

    fn next_required<'de, A, V>(&mut self, seq: &mut A) -> Result<V, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
        V: serde::Deserialize<'de>,
    {
        seq.next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(2, &"core value"))
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::{
        dif::cache::DIFSharedContainerCache,
        libs::core::core_lib_id::CoreLibIdIndex,
        values::{
            core_value::CoreValue,
            core_values::{
                decimal::typed_decimal::{DecimalTypeVariant, TypedDecimal},
                endpoint::Endpoint,
                integer::{Integer, typed_integer::IntegerTypeVariant},
                map::Map,
            },
        },
    };

    #[test]
    fn serialize_map() {
        // { endpoint: "@jonas" } -> [<map-idx>, { endpoint: [<endpoint-idx>, "@jonas"] }]
        let value =
            Value::from(CoreValue::Map(Map::StructuralWithStringKeys(vec![(
                "endpoint".into(),
                Value::from(Endpoint::from_str("@jonas").unwrap()).into(),
            )])));
        let mut cache = DIFSharedContainerCache::default();
        let serialized =
            SerdeContext::<Value>::new(&mut cache).serialize_to_json(&value);
        assert_eq!(
            serialized,
            format!(
                r#"[{},{{"endpoint":[{},"@jonas"]}}]"#,
                CoreLibIdIndex::from(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Map
                )),
                CoreLibIdIndex::from(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Endpoint
                ))
            )
        );

        // { "endpoint": "@jonas" } -> [<map-idx>, [[<endpoint-idx>, "@jonas"]]]
        let value = Value::from(CoreValue::Map(Map::Structural(vec![(
            "endpoint".into(),
            Value::from(Endpoint::from_str("@jonas").unwrap()).into(),
        )])));
        let serialized =
            SerdeContext::<Value>::new(&mut cache).serialize_to_json(&value);
        assert_eq!(
            serialized,
            format!(
                r#"[{},[["endpoint",[{},"@jonas"]]]]"#,
                CoreLibIdIndex::from(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Map
                )),
                CoreLibIdIndex::from(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Endpoint
                ))
            )
        );
    }

    #[test]
    fn default_representation() {
        // text
        let value = Value::from(CoreValue::Text("Hello, world!".into()));
        let serialized =
            SerdeContext::<Value>::new(&mut DIFSharedContainerCache::default())
                .serialize_to_json(&value);
        assert_eq!(serialized, r#""Hello, world!""#);

        // decimal f32
        let value = Value::from(CoreValue::TypedDecimal(TypedDecimal::F32(
            5.14f32.into(),
        )));
        let serialized =
            SerdeContext::<Value>::new(&mut DIFSharedContainerCache::default())
                .serialize_to_json(&value);
        assert_eq!(serialized, r#"5.14"#);

        // boolean
        let value = Value::from(CoreValue::Boolean(true.into()));
        let serialized =
            SerdeContext::<Value>::new(&mut DIFSharedContainerCache::default())
                .serialize_to_json(&value);
        assert_eq!(serialized, r#"true"#);
    }

    #[test]
    fn non_default_representation() {
        // 1 --> f32
        // 1 -> f32 -> [1, <nominal>]
        // 42 -> f32 -> [42, <nominal>]
        // 42 -> u8 [5, 42, <nominmal>]
        // ["integer/u8", 1] -> u8
        // [1, integer/u8]

        // f64
        let value = Value::from(CoreValue::TypedDecimal(TypedDecimal::F64(
            5.14f64.into(),
        )));
        let serialized =
            SerdeContext::<Value>::new(&mut DIFSharedContainerCache::default())
                .serialize_to_json(&value);

        assert_eq!(
            serialized,
            format!(
                r#"[{},5.14]"#,
                CoreLibIdIndex::from(CoreLibTypeId::Variant(
                    CoreLibVariantTypeId::Decimal(DecimalTypeVariant::F64)
                ))
            )
        );

        // integer
        let value = Value::from(CoreValue::Integer(Integer::new(42)));
        let serialized =
            SerdeContext::<Value>::new(&mut DIFSharedContainerCache::default())
                .serialize_to_json(&value);
        assert_eq!(
            serialized,
            format!(
                r#"[{},"42"]"#,
                CoreLibIdIndex::from(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Integer
                ))
            )
        );

        // typed integer
        let value = Value::from(CoreValue::TypedInteger(42u8.into()));
        let serialized =
            SerdeContext::<Value>::new(&mut DIFSharedContainerCache::default())
                .serialize_to_json(&value);
        assert_eq!(
            serialized,
            format!(
                r#"[{},42]"#,
                CoreLibIdIndex::from(CoreLibTypeId::Variant(
                    CoreLibVariantTypeId::Integer(IntegerTypeVariant::U8)
                ))
            )
        );
    }
}
