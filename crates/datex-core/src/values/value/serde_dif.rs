use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    values::{core_value::CoreValue, value::Value},
};
use serde::{Serialize, Serializer, ser::SerializeStruct};

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
        fn serialize_type_and_value_seed<'ctx2, 'borrow, Se, T>(
            ty: Cow<Option<Type>>,
            value: &T,
            serializer: Se,
            ctx: &'borrow mut SerdeContext<'ctx2, Value>,
        ) -> Result<Se::Ok, Se::Error>
        where
            T: Sized,
            Se: Serializer,
            for<'a> SerdeContext<'a, T>: SerializeSeed<Value = T>,
        {
            let mut state = serializer
                .serialize_struct("Value", if ty.is_some() { 2 } else { 1 })?;
            if let Some(ty) = ty.as_ref() {
                state.serialize_field(
                    "type",
                    &ValueWithSeed::new(ty, &mut ctx.cast::<Type>()),
                )?;
            }
            state.serialize_field(
                "value",
                &ValueWithSeed::new(value, &mut ctx.cast::<T>()),
            )?;
            state.end()
        }

        fn serialize_type_and_value<'ctx2, Se, T>(
            ty: Cow<Option<Type>>,
            value: &T,
            serializer: Se,
            ctx: &mut SerdeContext<'ctx2, Value>,
        ) -> Result<Se::Ok, Se::Error>
        where
            T: Serialize + Sized,
            Se: Serializer,
        {
            let mut state = serializer
                .serialize_struct("Value", if ty.is_some() { 2 } else { 1 })?;
            if let Some(ty) = ty.as_ref() {
                state.serialize_field(
                    "type",
                    &ValueWithSeed::new(ty, ctx.cast::<Type>()),
                )?;
            }
            state.serialize_field("value", value)?;
            state.end()
        }

        match &value.inner {
            CoreValue::Integer(i) => serialize_type_and_value(
                Cow::Borrowed(&value.custom_type),
                i,
                serializer,
                self,
            ),
            CoreValue::Boolean(b) => serialize_type_and_value(
                Cow::Borrowed(&value.custom_type),
                b,
                serializer,
                self,
            ),
            CoreValue::Text(s) => serialize_type_and_value(
                Cow::Borrowed(&value.custom_type),
                s,
                serializer,
                self,
            ),
            CoreValue::Decimal(d) => serialize_type_and_value(
                Cow::Borrowed(&value.custom_type),
                d,
                serializer,
                self,
            ),
            CoreValue::TypedInteger(ti) => serialize_type_and_value(
                Cow::Borrowed(&value.custom_type),
                ti,
                serializer,
                self,
            ),
            CoreValue::TypedDecimal(td) => serialize_type_and_value(
                Cow::Borrowed(&value.custom_type),
                td,
                serializer,
                self,
            ),
            CoreValue::Null => serialize_type_and_value(
                match &value.custom_type {
                    Some(_) => Cow::Borrowed(&value.custom_type),
                    None => {
                        Cow::Owned(Some(Type::core(CoreLibBaseTypeId::Null)))
                    }
                },
                &(),
                serializer,
                self,
            ),

            CoreValue::List(l) => serialize_type_and_value_seed(
                Cow::Borrowed(&value.custom_type),
                l,
                serializer,
                self,
            ),

            CoreValue::Range(range) => serialize_type_and_value_seed(
                match &value.custom_type {
                    Some(_) => Cow::Borrowed(&value.custom_type),
                    None => {
                        Cow::Owned(Some(Type::core(CoreLibBaseTypeId::Range)))
                    }
                },
                range,
                serializer,
                self,
            ),
            CoreValue::Endpoint(endpoint) => serialize_type_and_value(
                match &value.custom_type {
                    Some(_) => Cow::Borrowed(&value.custom_type),
                    None => Cow::Owned(Some(Type::core(
                        CoreLibBaseTypeId::Endpoint,
                    ))),
                },
                endpoint,
                serializer,
                self,
            ),

            CoreValue::Map(map_value) => serialize_type_and_value_seed(
                match &value.custom_type {
                    Some(_) => Cow::Borrowed(&value.custom_type),
                    None => {
                        Cow::Owned(Some(Type::core(CoreLibBaseTypeId::Map)))
                    }
                },
                map_value,
                serializer,
                self,
            ),

            // CoreValue::Type(t) => t.serialize(serializer),
            // CoreValue::Callable(c) => c.serialize(serializer),
            // CoreValue::NominalTypeDefinition(ntd) => ntd.serialize(serializer),
            _ => unimplemented!(
                "Serialization for this CoreValue variant is not implemented yet."
            ),
        }
    }
}

use crate::{libs::core::type_id::CoreLibBaseTypeId, types::r#type::Type};
use core::fmt;
use serde::{
    Deserializer,
    de::{DeserializeSeed, MapAccess, Visitor},
    ser::SerializeMap,
};

/// Deserialization for [Value] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Value> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // deserialize "value" property as CoreValue
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Value> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("struct Value with a 'value' property")
    }

    fn visit_map<A: MapAccess<'de>>(
        mut self,
        mut map: A,
    ) -> Result<Value, A::Error> {
        let mut core_value: Option<CoreValue> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "value" => {
                    core_value =
                        Some(map.next_value_seed(self.cast::<CoreValue>())?);
                }
                _ => {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
        }

        let core_value = core_value
            .ok_or_else(|| serde::de::Error::missing_field("value"))?;
        Ok(Value {
            inner: core_value,
            custom_type: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use log::info;

    use super::*;
    use crate::{
        dif::cache::DIFSharedContainerCache,
        values::{core_value::CoreValue, core_values::integer::Integer},
    };

    #[test]
    fn serialize_simple_local_value() {
        let value = Value::from(CoreValue::Integer(Integer::new(42)));
        let serialized =
            SerdeContext::<Value>::new(&mut DIFSharedContainerCache::default())
                .serialize_to_json(&value);

        assert_eq!(serialized, r#"{"value":"42"}"#);
    }
}
