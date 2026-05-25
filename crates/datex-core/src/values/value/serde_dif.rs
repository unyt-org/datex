use core::fmt;

use crate::{
    dif::serde_context::SerdeContext,
    libs::core::{
        core_lib_id::CoreLibId,
        type_id::{CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId},
    },
    prelude::*,
    types::type_definition::TypeDefinition,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    values::{
        core_value::CoreValue,
        core_values::{
            decimal::typed_decimal::TypedDecimal,
            integer::typed_integer::TypedInteger,
        },
        value::Value,
    },
};
use serde::{Serialize, Serializer, de::Visitor, ser::SerializeStruct};

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
        fn serialize_custom_or_direct<'ctx2, Se, T>(
            custom_type: &Option<TypeDefinition>,
            inner: &T,
            serializer: Se,
            ctx: &mut SerdeContext<'ctx2, Value>,
        ) -> Result<Se::Ok, Se::Error>
        where
            T: Serialize + Sized,
            Se: Serializer,
        {
            use serde::ser::SerializeTuple;

            match custom_type {
                Some(custom_type) => {
                    // [custom_type, value, custom_type_definition]
                    let mut tuple = serializer.serialize_tuple(3)?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        custom_type,
                        &mut ctx.cast::<TypeDefinition>(),
                    ))?;

                    tuple.serialize_element(inner)?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        custom_type,
                        &mut ctx.cast::<TypeDefinition>(),
                    ))?;

                    tuple.end()
                }

                None => inner.serialize(serializer),
            }
        }

        fn serialize_custom_or_direct_seed<'ctx2, 'borrow, Se, T>(
            custom_type: &Option<TypeDefinition>,
            inner: &T,
            serializer: Se,
            ctx: &'borrow mut SerdeContext<'ctx2, Value>,
        ) -> Result<Se::Ok, Se::Error>
        where
            T: Sized,
            Se: Serializer,
            for<'a> SerdeContext<'a, T>: SerializeSeed<Value = T>,
        {
            use serde::ser::SerializeTuple;

            match custom_type {
                Some(custom_type) => {
                    // [custom_type, value, custom_type_definition]
                    let mut tuple = serializer.serialize_tuple(3)?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        custom_type,
                        &mut ctx.cast::<TypeDefinition>(),
                    ))?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        inner,
                        &mut ctx.cast::<T>(),
                    ))?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        custom_type,
                        &mut ctx.cast::<TypeDefinition>(),
                    ))?;

                    tuple.end()
                }

                None => ValueWithSeed::new(inner, &mut ctx.cast::<T>())
                    .serialize(serializer),
            }
        }

        fn serialize_custom_or_typed<'ctx2, Se, T>(
            custom_type: &Option<TypeDefinition>,
            ty: TypeDefinition,
            inner: &T,
            serializer: Se,
            ctx: &mut SerdeContext<'ctx2, Value>,
        ) -> Result<Se::Ok, Se::Error>
        where
            T: Serialize + Sized,
            Se: Serializer,
        {
            use serde::ser::SerializeTuple;

            match custom_type {
                Some(custom_type) => {
                    // [custom_type, value, custom_type_definition]
                    let mut tuple = serializer.serialize_tuple(3)?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        custom_type,
                        &mut ctx.cast::<TypeDefinition>(),
                    ))?;

                    tuple.serialize_element(inner)?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        custom_type,
                        &mut ctx.cast::<TypeDefinition>(),
                    ))?;

                    tuple.end()
                }

                None => {
                    // [type, value]
                    let mut tuple = serializer.serialize_tuple(2)?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        &ty,
                        &mut ctx.cast::<TypeDefinition>(),
                    ))?;

                    tuple.serialize_element(inner)?;

                    tuple.end()
                }
            }
        }

        fn serialize_custom_or_typed_seed<'ctx2, 'borrow, Se, T>(
            custom_type: &Option<TypeDefinition>,
            ty: TypeDefinition,
            inner: &T,
            serializer: Se,
            ctx: &'borrow mut SerdeContext<'ctx2, Value>,
        ) -> Result<Se::Ok, Se::Error>
        where
            T: Sized,
            Se: Serializer,
            for<'a> SerdeContext<'a, T>: SerializeSeed<Value = T>,
        {
            use serde::ser::SerializeTuple;

            match custom_type {
                Some(custom_type) => {
                    // [custom_type, value, custom_type_definition]
                    let mut tuple = serializer.serialize_tuple(3)?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        custom_type,
                        &mut ctx.cast::<TypeDefinition>(),
                    ))?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        inner,
                        &mut ctx.cast::<T>(),
                    ))?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        custom_type,
                        &mut ctx.cast::<TypeDefinition>(),
                    ))?;

                    tuple.end()
                }

                None => {
                    // [type, value]
                    let mut tuple = serializer.serialize_tuple(2)?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        &ty,
                        &mut ctx.cast::<TypeDefinition>(),
                    ))?;

                    tuple.serialize_element(&ValueWithSeed::new(
                        inner,
                        &mut ctx.cast::<T>(),
                    ))?;

                    tuple.end()
                }
            }
        }

        match &value.inner {
            // Can be serialized directly if there is no custom type.
            CoreValue::Boolean(b) => serialize_custom_or_direct(
                &value.custom_type,
                b,
                serializer,
                self,
            ),

            CoreValue::Text(s) => serialize_custom_or_direct(
                &value.custom_type,
                s,
                serializer,
                self,
            ),

            CoreValue::Decimal(d) => serialize_custom_or_direct(
                &value.custom_type,
                d,
                serializer,
                self,
            ),

            CoreValue::Null => serialize_custom_or_direct(
                &value.custom_type,
                &(),
                serializer,
                self,
            ),

            CoreValue::List(l) => serialize_custom_or_direct_seed(
                &value.custom_type,
                l,
                serializer,
                self,
            ),

            // Plain integer loses exact intent in JSON-like formats, so keep type info.
            CoreValue::Integer(i) => serialize_custom_or_typed(
                &value.custom_type,
                TypeDefinition::Core(CoreLibBaseTypeId::Integer.into()),
                i,
                serializer,
                self,
            ),

            CoreValue::TypedInteger(ti) => serialize_custom_or_typed(
                &value.custom_type,
                TypeDefinition::Core(CoreLibTypeId::Variant(
                    CoreLibVariantTypeId::Integer(ti.variant()),
                )),
                ti,
                serializer,
                self,
            ),

            CoreValue::TypedDecimal(td) => match td {
                TypedDecimal::F32(_) => serialize_custom_or_direct(
                    &value.custom_type,
                    td,
                    serializer,
                    self,
                ),

                TypedDecimal::F64(_) => serialize_custom_or_typed(
                    &value.custom_type,
                    TypeDefinition::Core(CoreLibTypeId::Variant(
                        CoreLibVariantTypeId::Decimal(td.variant()),
                    )),
                    td,
                    serializer,
                    self,
                ),

                TypedDecimal::Decimal(_) => serialize_custom_or_typed(
                    &value.custom_type,
                    TypeDefinition::Core(CoreLibBaseTypeId::Decimal.into()),
                    td,
                    serializer,
                    self,
                ),
            },

            CoreValue::Range(range) => serialize_custom_or_typed_seed(
                &value.custom_type,
                TypeDefinition::Core(CoreLibBaseTypeId::Range.into()),
                range,
                serializer,
                self,
            ),

            CoreValue::Endpoint(endpoint) => serialize_custom_or_typed(
                &value.custom_type,
                TypeDefinition::Core(CoreLibBaseTypeId::Endpoint.into()),
                endpoint,
                serializer,
                self,
            ),

            CoreValue::Map(map_value) => serialize_custom_or_typed_seed(
                &value.custom_type,
                TypeDefinition::Core(CoreLibBaseTypeId::Map.into()),
                map_value,
                serializer,
                self,
            ),

            _ => unimplemented!(
                "Serialization for this CoreValue variant is not implemented yet."
            ),
        }
    }
}
impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Value> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("struct Value with a 'value' property")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // the value can be a array with length 3, where first element is the custom type
        // the second element is the core value and the third element is the custom type definition for the custom type,
        // or it can be an array with length 2, where the first element is the core value and the second element is the custom type definition
        // for the custom type

        let mut custom_type: Option<TypeDefinition> = None;
        let mut core_value: Option<CoreValue> = None;

        let len = seq.size_hint().ok_or_else(|| {
            serde::de::Error::custom(
                "Value sequence must have known length: expected length 2 or 3",
            )
        })?;
        match len {
            2 => {
                use alloc::borrow::Cow;

                let raw =
                    seq.next_element::<Cow<'de, str>>()?.ok_or_else(|| {
                        serde::de::Error::invalid_length(0, &self)
                    })?;

                let custom_type_definition =
                    CoreLibId::try_from_str(raw.as_ref()).ok_or_else(|| {
                        serde::de::Error::custom(format!(
                            "invalid core lib id: {raw:?}"
                        ))
                    })?;

                let inner =
                    seq.next_element_seed(self.cast::<CoreValue>())?.unwrap();

                Ok(Value {
                    custom_type: None,
                    inner,
                })
            }

            3 => {
                todo!()
            }

            other => Err(serde::de::Error::invalid_length(other, &self)),
        }
    }

    // fn visit_map<A: MapAccess<'de>>(
    //     mut self,
    //     mut map: A,
    // ) -> Result<Value, A::Error> {
    //     while let Some(key) = map.next_key::<String>()? {
    //         match key.as_str() {
    //             "custom_type" => {
    //                 todo!("custom type")
    //             }
    //             _ => {
    //                 map.next_value::<serde::de::IgnoredAny>()?;
    //             }
    //         }
    //     }

    //     let core_value = self.cast::<CoreValue>().visit_map(map)?;

    //     Ok(Value {
    //         inner: core_value,
    //         custom_type: None,
    //     })
    // }
}

#[cfg(test)]
mod tests {
    use log::info;

    use super::*;
    use crate::{
        dif::cache::DIFSharedContainerCache,
        values::{
            core_value::CoreValue,
            core_values::{
                decimal::typed_decimal::TypedDecimal, integer::Integer,
            },
        },
    };

    #[test]
    fn serialize_map() {
        /*
        { type: "map", value: [
           [{type: "text", value: "endpoint"}, {type: "endpoint", value: "@jonas"}]
        ], custom_type: "sdf"})
        */
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
    }

    #[test]
    fn non_default_representation() {
        let value = Value::from(CoreValue::TypedDecimal(TypedDecimal::F64(
            5.14f64.into(),
        )));
        let serialized =
            SerdeContext::<Value>::new(&mut DIFSharedContainerCache::default())
                .serialize_to_json(&value);
        println!("Serialized value: {serialized}");
        
        // 1 --> f32
        // 1 -> f32 -> [1, <nominal>]
        // 42 -> f32 -> [42, <nominal>]
        // 42 -> u8 [5, 42, <nominmal>]
        // ["integer/u8", 1] -> u8
        // [1, integer/u8]
        assert_eq!(
            serialized,
            r#"[["integer/u8"],5.14]"#
        );
    }

    #[test]
    fn serialize_simple_local_value() {
        let value = Value::from(CoreValue::Integer(Integer::new(42)));
        let serialized =
            SerdeContext::<Value>::new(&mut DIFSharedContainerCache::default())
                .serialize_to_json(&value);
        println!("Serialized value: {serialized}");
        assert_eq!(serialized, r#"{"value":"42"}"#);
    }
}
