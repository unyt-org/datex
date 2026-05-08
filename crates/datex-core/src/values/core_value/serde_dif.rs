use crate::{
    prelude::*,
    values::{
        core_value::CoreValue,
        core_values::{endpoint::Endpoint, list::List, map::Map, range::Range},
        value_container::ValueContainer,
    },
};
use serde::{Serialize, Serializer, de::MapAccess, ser::SerializeMap};

use crate::dif::serde_context::SerdeContext;
use core::fmt;
use serde::{
    Deserializer,
    de::{DeserializeSeed, SeqAccess, Visitor},
};
/// Deserialization for [CoreValue] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, CoreValue> {
    type Value = CoreValue;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<CoreValue, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, CoreValue> {
    type Value = CoreValue;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a CoreValue")
    }

    fn visit_bool<E>(self, value: bool) -> Result<CoreValue, E> {
        Ok(CoreValue::Boolean(value.into()))
    }

    fn visit_i64<E>(self, value: i64) -> Result<CoreValue, E> {
        Ok(CoreValue::Integer(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<CoreValue, E> {
        Ok(CoreValue::Integer(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<CoreValue, E> {
        Ok(CoreValue::Decimal(value.into()))
    }

    fn visit_str<E>(self, value: &str) -> Result<CoreValue, E>
    where
        E: serde::de::Error,
    {
        Ok(CoreValue::Text(value.into()))
    }

    fn visit_string<E>(self, value: String) -> Result<CoreValue, E> {
        Ok(CoreValue::Text(value.into()))
    }

    fn visit_seq<A: SeqAccess<'de>>(
        mut self,
        mut seq: A,
    ) -> Result<CoreValue, A::Error> {
        let mut items = Vec::new();
        while let Some(item) =
            seq.next_element_seed(self.cast::<ValueContainer>())?
        {
            items.push(item);
        }
        Ok(CoreValue::List(List::from(items)))
    }

    // fn visit_map<A>(mut self, mut map: A) -> Result<CoreValue, A::Error>
    // where
    //     A: MapAccess<'de>,
    // {
    //     let mut items = Vec::new();

    //     while let Some(key) = {
    //         let key_seed = self.cast::<ValueContainer>();
    //         map.next_key_seed(key_seed)?
    //     } {
    //         let value = {
    //             let value_seed = self.cast::<ValueContainer>();
    //             map.next_value_seed(value_seed)?
    //         };

    //         items.push((key, value));
    //     }

    //     Ok(CoreValue::Map(Map::from(items)))
    // }
    fn visit_map<A>(mut self, mut map: A) -> Result<CoreValue, A::Error>
    where
        A: MapAccess<'de>,
    {
        use serde::de::Error;

        let mut ty: Option<String> = None;
        let mut out: Option<CoreValue> = None;

        while let Some(field) = map.next_key::<String>()? {
            match field.as_str() {
                "$type" => {
                    ty = Some(map.next_value()?);
                }

                "value" => {
                    let ty = ty.as_deref().ok_or_else(|| {
                        A::Error::custom("`$type` must come before `value`")
                    })?;

                    let value = match ty {
                        "map" => {
                            let value =
                                map.next_value_seed(self.cast::<Map>())?;
                            CoreValue::Map(value)
                        }

                        "range" => {
                            let value =
                                map.next_value_seed(self.cast::<Range>())?;
                            CoreValue::Range(value)
                        }

                        "endpoint" => {
                            let value =
                                map.next_value_seed(self.cast::<Endpoint>())?;
                            CoreValue::Endpoint(value)
                        }

                        other => {
                            return Err(A::Error::custom(format!(
                                "unknown CoreValue `$type`: {other}"
                            )));
                        }
                    };

                    out = Some(value);
                }

                other => {
                    return Err(A::Error::custom(format!(
                        "unexpected field `{other}` in CoreValue object"
                    )));
                }
            }
        }

        out.ok_or_else(|| {
            A::Error::custom("missing `value` field in CoreValue object")
        })
    }
}
