use crate::{
    dif::serde_context::SerdeContext,
    libs::core::{core_lib_id::CoreLibIdIndex, type_id::CoreLibTypeId},
    prelude::*,
    types::type_definition::TypeDefinition,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    values::{
        core_value::{CoreValue, serde_dif::CoreValueVisitor},
        core_values::{
            boolean::Boolean, decimal::typed_decimal::TypedDecimal,
            native::NativeCoreValue,
        },
        value::Value,
        value_container::ValueContainer,
    },
};
use core::fmt;
use num::ToPrimitive;
use serde::{
    Deserializer, Serialize, Serializer,
    de::{DeserializeSeed, Error as DeError, Visitor},
    ser::SerializeTuple,
};

impl<'ctx> SerdeContext<'ctx, Value> {
    /// This method is used to serialize a value that can be represented directly depending on the flag set (e.g. a boolean or a text)
    /// or with a custom type definition (e.g. a nominal type).
    /// ## For no custom type:
    ///   true -> "true"
    ///   "Hello" -> "Hello"
    ///   42f64 -> 42
    ///   42f32 -> "42"
    /// ## For custom type:
    /// {custom_type: LiteralTypeDefinition::Integer(42), value: 42} -> [<core_lib_id>, <type_definition>, 42]
    fn serialize_with_core_type<Se, T>(
        &mut self,
        inner: &T,
        core_lib_type_id: CoreLibTypeId,
        custom_type: &Option<TypeDefinition>,
        serializer: Se,
        direct: bool,
    ) -> Result<Se::Ok, Se::Error>
    where
        T: Serialize + ?Sized,
        Se: Serializer,
    {
        if direct && custom_type.is_none() {
            return inner.serialize(serializer);
        }

        let index = CoreLibIdIndex::from(core_lib_type_id);
        let mut tuple = serializer
            .serialize_tuple(if custom_type.is_some() { 3 } else { 2 })?;
        tuple.serialize_element(&index.to_u16())?;
        tuple.serialize_element(inner)?;
        if let Some(custom_type) = custom_type {
            // [id, value, custom_type]
            tuple.serialize_element(&ValueWithSeed::new(
                custom_type,
                &mut self.cast::<TypeDefinition>(),
            ))?;
        } else {
            // [id, value]
        }
        tuple.end()
    }

    fn serialize_with_core_type_serde<Se, T>(
        &mut self,
        inner: &T,
        core_lib_type_id: CoreLibTypeId,
        custom_type: &Option<TypeDefinition>,
        serializer: Se,
        direct: bool,
    ) -> Result<Se::Ok, Se::Error>
    where
        T: Sized,
        Se: Serializer,
        for<'a> SerdeContext<'a, T>: SerializeSeed<Value = T>,
    {
        if direct && custom_type.is_none() {
            return self.cast::<T>().serialize(inner, serializer);
        }
        let index = CoreLibIdIndex::from(core_lib_type_id);
        let mut tuple = serializer
            .serialize_tuple(if custom_type.is_some() { 3 } else { 2 })?;
        tuple.serialize_element(&index.to_u16())?;
        tuple.serialize_element(&ValueWithSeed::new(
            inner,
            &mut self.cast::<T>(),
        ))?;
        if let Some(custom_type) = custom_type {
            // [id, value, custom_type]
            tuple.serialize_element(&ValueWithSeed::new(
                custom_type,
                &mut self.cast::<TypeDefinition>(),
            ))?;
        } else {
            // [id, value]
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
        let core_lib_type = value.default_core_type();
        match &value.inner {
            // Direct serializable core values, that can be serialized as they can be unambiguously deserialized without it
            CoreValue::Boolean(b) => self.serialize_with_core_type(
                b,
                core_lib_type,
                &value.custom_type,
                serializer,
                true,
            ),
            CoreValue::Text(s) => self.serialize_with_core_type(
                s,
                core_lib_type,
                &value.custom_type,
                serializer,
                true,
            ),
            CoreValue::Null => self.serialize_with_core_type(
                &(),
                core_lib_type,
                &value.custom_type,
                serializer,
                true,
            ),
            CoreValue::TypedDecimal(dec @ TypedDecimal::F64(_)) => self
                .serialize_with_core_type(
                    &dec,
                    core_lib_type,
                    &value.custom_type,
                    serializer,
                    dec.is_finite(),
                ),

            // Core values that require a specific core type id to be serialized for non-ambiguous deserialization
            CoreValue::Endpoint(endpoint) => self.serialize_with_core_type(
                endpoint,
                core_lib_type,
                &value.custom_type,
                serializer,
                false,
            ),
            CoreValue::Decimal(d) => self.serialize_with_core_type(
                d,
                core_lib_type,
                &value.custom_type,
                serializer,
                false,
            ),
            CoreValue::Integer(i) => self.serialize_with_core_type(
                i,
                core_lib_type,
                &value.custom_type,
                serializer,
                false,
            ),
            CoreValue::TypedInteger(ti) => self.serialize_with_core_type(
                ti,
                core_lib_type,
                &value.custom_type,
                serializer,
                false,
            ),
            CoreValue::TypedDecimal(td) => self.serialize_with_core_type(
                td,
                core_lib_type,
                &value.custom_type,
                serializer,
                false,
            ),

            // Complex core values, that can contain nested values
            CoreValue::List(l) => self.serialize_with_core_type_serde(
                l,
                core_lib_type,
                &value.custom_type,
                serializer,
                false,
            ),
            CoreValue::Range(range) => self.serialize_with_core_type_serde(
                range,
                core_lib_type,
                &value.custom_type,
                serializer,
                false,
            ),

            CoreValue::Map(map) => self.serialize_with_core_type_serde(
                map,
                core_lib_type,
                &value.custom_type,
                serializer,
                false,
            ),
            CoreValue::Type(ty) => self.serialize_with_core_type_serde(
                ty,
                core_lib_type,
                &value.custom_type,
                serializer,
                false,
            ),
            CoreValue::EntityTypeDefinition(_entity_type_definition) => {
                todo!()
            }
            CoreValue::Callable(callable) => self
                .serialize_with_core_type_serde(
                    callable,
                    core_lib_type,
                    &value.custom_type,
                    serializer,
                    false,
                ),
            CoreValue::Box(inner) => {
                self.cast::<ValueContainer>().serialize(inner, serializer)
            }
            CoreValue::Uninitialized => panic!("Uninitialized value"),
            CoreValue::Native(native) => {
                self.cast::<NativeCoreValue>().serialize(native, serializer)
            }
        }
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Value> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a value, which can be a direct core value (e.g. boolean, text) or a complex value with a custom type definition")
    }

    /// default mapping for unit: null
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(Value::from(CoreValue::Null))
    }

    /// default mapping for none: null
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_unit()
    }

    /// default mapping for bool: boolean
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(Value::from(CoreValue::Boolean(Boolean::new(v))))
    }

    /// default mapping for string: text
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(Value::from(CoreValue::Text(v.into())))
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_str(&v)
    }

    /// default mapping for f64: decimal/f64
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(Value::from(CoreValue::TypedDecimal(TypedDecimal::F64(
            v.into(),
        ))))
    }

    // default mapping for integers: decimal/f64 (with a check for overflow)
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v as f64)
    }
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v as f64)
    }
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v as f64)
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v.to_f64().ok_or_else(|| {
            DeError::custom(format!(
                "i64 value {v} is too large to fit into f64"
            ))
        })?)
    }
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v as f64)
    }
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v as f64)
    }
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v as f64)
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v.to_f64().ok_or_else(|| {
            DeError::custom(format!(
                "u64 value {v} is too large to fit into f64"
            ))
        })?)
    }
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v as f64)
    }
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v.to_f64().ok_or_else(|| {
            DeError::custom(format!(
                "i128 value {v} is too large to fit into f64"
            ))
        })?)
    }
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_f64(v.to_f64().ok_or_else(|| {
            DeError::custom(format!(
                "u128 value {v} is too large to fit into f64"
            ))
        })?)
    }

    /// mapping for [core_lib_type_id, value, custom_type?]
    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let core_lib_type_id: u16 = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::custom("expected a sequence with at least one element for core value deserialization")
        })?;

        let core_lib_id = CoreLibTypeId::try_from(CoreLibIdIndex(
            core_lib_type_id,
        ))
        .map_err(|_| {
            serde::de::Error::custom("invalid core lib id index".to_string())
        })?;

        let visitor = CoreValueVisitor {
            core_lib_id,
            context: &mut self,
        };
        let inner: CoreValue = seq.next_element_seed(visitor)?.ok_or_else(|| {
            serde::de::Error::custom(format!(
                "expected a sequence with at least two elements for core value deserialization, got only one (core lib id: {core_lib_id})"
            ))
        })?;

        let custom_type: Option<TypeDefinition> =
            seq.next_element_seed(self.cast::<TypeDefinition>()).map_err(|err| {
                serde::de::Error::custom(format!(
                    "error deserializing custom type definition for core value: {err}"
                ))
            })?;

        Ok(Value::new(inner, custom_type))
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Value> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        libs::core::{
            core_lib_id::CoreLibIdIndex,
            type_id::{CoreLibBaseTypeId, CoreLibVariantTypeId},
        },
        runtime::cache::shared_values_cache::SharedValuesCache,
        values::{
            core_value::CoreValue,
            core_values::{
                decimal::typed_decimal::{DecimalTypeVariant, TypedDecimal},
                endpoint::Endpoint,
                integer::{Integer, typed_integer::IntegerTypeVariant},
                map::{Map, MapEntries},
            },
            value_container::ValueContainer,
        },
    };
    use core::str::FromStr;
    use test_case::test_case;

    #[test]
    fn endpoint_serialization() {
        let endpoint = Endpoint::from_str("@jonas").unwrap();
        let value = Value::new(CoreValue::Endpoint(endpoint.clone()), None);
        let mut cache = SharedValuesCache::default();
        let serialized =
            SerdeContext::<Value>::new(&mut cache).serialize_to_json(&value);
        assert_eq!(
            serialized,
            format!(
                r#"[{},"{}"]"#,
                CoreLibIdIndex::from(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Endpoint
                )),
                endpoint
            )
        );
    }

    #[test]
    fn serialize_map() {
        let mut cache = SharedValuesCache::default();

        // { endpoint: "@jonas" } -> [<map-idx>, { endpoint: [<endpoint-idx>, "@jonas"] }]
        let value = Value::from(CoreValue::Map(
            Map::structural_with_string_keys(vec![(
                "endpoint".into(),
                ValueContainer::Local(Value::from(
                    Endpoint::from_str("@jonas").unwrap(),
                )),
            )]),
        ));
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
        let value = Value::from(CoreValue::Map(Map::structural(vec![(
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
            SerdeContext::<Value>::new(&mut SharedValuesCache::default())
                .serialize_to_json(&value);
        assert_eq!(serialized, r#""Hello, world!""#);

        // decimal f64
        let value = Value::from(CoreValue::TypedDecimal(TypedDecimal::F64(
            5.14f64.into(),
        )));
        let serialized =
            SerdeContext::<Value>::new(&mut SharedValuesCache::default())
                .serialize_to_json(&value);
        assert_eq!(serialized, r#"5.14"#);

        // boolean
        let value = Value::from(CoreValue::Boolean(true.into()));
        let serialized =
            SerdeContext::<Value>::new(&mut SharedValuesCache::default())
                .serialize_to_json(&value);
        assert_eq!(serialized, r#"true"#);
    }

    #[test]
    fn non_default_representation() {
        // f32
        let value = Value::from(CoreValue::TypedDecimal(TypedDecimal::F32(
            5.14f32.into(),
        )));
        let serialized =
            SerdeContext::<Value>::new(&mut SharedValuesCache::default())
                .serialize_to_json(&value);

        assert_eq!(
            serialized,
            format!(
                r#"[{},5.14]"#,
                CoreLibIdIndex::from(CoreLibTypeId::Variant(
                    CoreLibVariantTypeId::Decimal(DecimalTypeVariant::F32)
                ))
            )
        );

        // integer
        let value = Value::from(CoreValue::Integer(Integer::new(42)));
        let serialized =
            SerdeContext::<Value>::new(&mut SharedValuesCache::default())
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
            SerdeContext::<Value>::new(&mut SharedValuesCache::default())
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

    #[test_case(
        CoreValue::Text("Hello, world!".into()) ; "text"
    )]
    #[test_case(
        CoreValue::TypedDecimal(TypedDecimal::F32(5.14f32.into())) ; "decimal f32"
    )]
    #[test_case(
        CoreValue::TypedDecimal(TypedDecimal::F64(f64::NAN.into())) ; "nan f64"
    )]
    #[test_case(
        CoreValue::TypedDecimal(TypedDecimal::F32(f32::NAN.into())) ; "nan f32"
    )]
    #[test_case(
        CoreValue::TypedDecimal(TypedDecimal::F32(f32::INFINITY.into())) ; "inf f32"
    )]
    #[test_case(
        CoreValue::TypedDecimal(TypedDecimal::F64(f64::INFINITY.into())) ; "inf f64"
    )]
    #[test_case(
        CoreValue::TypedDecimal(TypedDecimal::F64(5.14f64.into())) ; "decimal f64"
    )]
    #[test_case(
        CoreValue::Boolean(true.into()) ; "boolean"
    )]
    #[test_case(
        CoreValue::TypedInteger(42u8.into()) ; "typed integer u8"
    )]
    #[test_case(
        CoreValue::Endpoint(Endpoint::from_str("@jonas").unwrap()) ; "endpoint"
    )]
    #[test_case(
        CoreValue::Map(Map::structural_with_string_keys(vec![(
            "endpoint".into(),
            ValueContainer::Local(Value::from(Endpoint::from_str("@jonas").unwrap())),
        )])) ; "map with string keys"
    )]
    #[test_case(
        CoreValue::Null ; "null"
    )]
    #[test_case(
        CoreValue::Integer(Integer::new(42)) ; "base integer"
    )]
    #[test_case(
        CoreValue::TypedInteger((-42i8).into()) ; "typed integer i8"
    )]
    #[test_case(
        CoreValue::TypedInteger(42u16.into()) ; "typed integer u16"
    )]
    #[test_case(
        CoreValue::TypedInteger(42u32.into()) ; "typed integer u32"
    )]
    #[test_case(
        CoreValue::TypedInteger(42u64.into()) ; "typed integer u64"
    )]
    #[test_case(
        CoreValue::TypedInteger(42u128.into()) ; "typed integer u128"
    )]
    #[test_case(
        CoreValue::TypedDecimal(TypedDecimal::F64(f64::NEG_INFINITY.into())) ; "negative inf f64"
    )]
    #[test_case(
        CoreValue::TypedDecimal(TypedDecimal::F32(f32::NEG_INFINITY.into())) ; "negative inf f32"
    )]
    fn roundtrip_no_custom_type(value: CoreValue) {
        let value = Value::from(value);
        let mut cache = SharedValuesCache::default();
        let serialized =
            SerdeContext::<Value>::new(&mut cache).serialize_to_json(&value);
        let deserialized: Value = SerdeContext::<Value>::new(&mut cache)
            .try_deserialize_from_json(&serialized)
            .unwrap();
        assert_eq!(deserialized, value);
    }
}
