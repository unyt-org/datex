use core::{fmt, fmt::Display, panic};

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
        value_container::ValueContainer,
    },
};
use num::ToPrimitive;
use ordered_float::OrderedFloat;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{DeserializeSeed, Visitor},
    forward_to_deserialize_any,
    ser::{SerializeMap, SerializeStruct, SerializeTuple},
};

impl<'ctx> SerdeContext<'ctx, Value> {
    /// This method is used to serialize a value that can be represented directly depending on the flag set (e.g. a boolean or a text)
    /// or with a custom type definition (e.g. a nominal type).
    /// ## For no custom type:
    ///   true -> "true"
    ///   "Hello" -> "Hello"
    ///   42f32 -> 42
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
            CoreValue::TypedDecimal(TypedDecimal::F64(OrderedFloat(f64))) => {
                self.serialize_with_core_type(
                    f64,
                    core_lib_type,
                    &value.custom_type,
                    serializer,
                    true,
                )
            }
            CoreValue::Map(Map::StructuralWithStringKeys(map)) => {
                let mut map_serializer =
                    serializer.serialize_map(Some(map.len()))?;
                for (key, value) in map {
                    map_serializer.serialize_key(key)?;
                    map_serializer.serialize_value(&ValueWithSeed::new(
                        value,
                        self.cast::<ValueContainer>(),
                    ))?;
                }
                map_serializer.end()
            }

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
            CoreValue::Type(ty) => todo!(),
            CoreValue::NominalTypeDefinition(nominal_type_definition) => {
                todo!()
            }
            CoreValue::Callable(callable) => todo!(),
        }
    }
}

impl<'ctx> SerdeContext<'ctx, Value> {
    fn visit_number<E: serde::de::Error, T: ToPrimitive + Display>(
        self,
        v: T,
    ) -> Result<Value, E> {
        let v: f64 = v.to_f64().ok_or_else(|| {
            serde::de::Error::custom(format!(
                "integer value {v} is out of range for f64"
            ))
        })?;
        if !v.is_finite() {
            return Err(E::custom(format!(
                "invalid floating point value: {v}"
            )));
        }
        Ok(Value {
            inner: CoreValue::TypedDecimal(TypedDecimal::F64(OrderedFloat(v))),
            custom_type: None,
        })
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Value> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a value, which can be a direct core value (e.g. boolean, text) or a complex value with a custom type definition")
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
        self.visit_number(v)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
    }

    // Rationale: JSON only knowns 'numbers', so we have to implement all visitors, as whichever JSON implementation
    // is used to serialize / deserialize DIF, we don't know, which visitor is getting called.
    // The following methods for all integer and floating point types, all deserialize as F64
    // Attention: We can map the whole range of f32
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
    }
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
    }
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
    }
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
    }
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
    }
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
    }
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
    }
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
    }
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_number(v)
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
        let core_lib_type_id: u16 = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::custom("expected a sequence with at least one element for core value deserialization")
        })?;

        let core_lib_id = CoreLibTypeId::try_from(CoreLibIdIndex(
            core_lib_type_id,
        ))
        .map_err(|_| {
            serde::de::Error::custom("invalid core lib id index".to_string())
        })?;

        let inner: CoreValue =
            self.next_core_value_by_type_id(&mut seq, core_lib_id)?;

        // FIXME Why the option not works??
        let custom_type: Option<TypeDefinition> =
            seq.next_element_seed(self.cast::<TypeDefinition>()).map_err(|err| {
                serde::de::Error::custom(format!(
                    "error deserializing custom type definition for core value: {err}"
                ))
            })?;

        Ok(Value { custom_type, inner })
    }
}

impl<'ctx, T> SerdeContext<'ctx, T> {
    fn next_core_value_by_type_id<'de, A>(
        &mut self,
        seq: &mut A,
        core_lib_type_id: CoreLibTypeId,
    ) -> Result<CoreValue, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        match core_lib_type_id {
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
            value_container::ValueContainer,
        },
    };
    use core::str::FromStr;
    use test_case::test_case;

    #[test]
    fn endpoint_serialization() {
        let endpoint = Endpoint::from_str("@jonas").unwrap();
        let value = Value {
            inner: CoreValue::Endpoint(endpoint.clone()),
            custom_type: None,
        };
        let mut cache = DIFSharedContainerCache::default();
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
        // { endpoint: "@jonas" } -> [<map-idx>, { endpoint: [<endpoint-idx>, "@jonas"] }]
        let value =
            Value::from(CoreValue::Map(Map::StructuralWithStringKeys(vec![(
                "endpoint".into(),
                ValueContainer::Local(Value::from(
                    Endpoint::from_str("@jonas").unwrap(),
                )),
            )])));
        let mut cache = DIFSharedContainerCache::default();
        let serialized =
            SerdeContext::<Value>::new(&mut cache).serialize_to_json(&value);
        assert_eq!(
            serialized,
            format!(
                r#"{{"endpoint":[{},"@jonas"]}}"#,
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

    #[test_case(
        CoreValue::Text("Hello, world!".into()) ; "text"
    )]
    #[test_case(
        CoreValue::TypedDecimal(TypedDecimal::F32(5.14f32.into())) ; "decimal f32"
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
        CoreValue::Map(Map::StructuralWithStringKeys(vec![(
            "endpoint".into(),
            ValueContainer::Local(Value::from(Endpoint::from_str("@jonas").unwrap())),
        )])) ; "map with string keys"
    )]
    fn roundtrip_no_custom_type(value: CoreValue) {
        let value = Value::from(value);
        let mut cache = DIFSharedContainerCache::default();
        let serialized =
            SerdeContext::<Value>::new(&mut cache).serialize_to_json(&value);
        let deserialized: Value = SerdeContext::<Value>::new(&mut cache)
            .try_deserialize_from_json(&serialized)
            .unwrap();
        assert_eq!(deserialized, value);
    }
}
