use core::fmt;

use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    values::{core_values::range::Range, value_container::ValueContainer},
};
use serde::{
    Serialize, Serializer,
    de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor},
    ser::SerializeStruct,
};

impl Serialize for Range {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Range", 2)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Range> {
    type Value = Range;

    fn deserialize<D>(self, deserializer: D) -> Result<Range, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Range> {
    type Value = Range;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a range")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Range, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut start: Option<ValueContainer> = None;
        let mut end: Option<ValueContainer> = None;
        while let Some(field) = map.next_key::<String>()? {
            match field.as_str() {
                "start" => {
                    start = Some(
                        map.next_value_seed(self.cast::<ValueContainer>())?,
                    );
                }

                "end" => {
                    end = Some(
                        map.next_value_seed(self.cast::<ValueContainer>())?,
                    );
                }
                _ => {
                    let _: IgnoredAny = map.next_value()?;
                }
            }
        }

        let start =
            start.ok_or_else(|| A::Error::custom("missing start field"))?;
        let end = end.ok_or_else(|| A::Error::custom("missing end field"))?;

        Ok(Range {
            start: Box::new(start),
            end: Box::new(end),
        })
    }
}
