use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    values::{core_values::list::List, value_container::ValueContainer},
};
use serde::{
    Serializer,
    ser::{SerializeMap, SerializeSeq},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, List> {
    type Value = List;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_seq(Some(value.len() as usize))?;
        for value in value.iter() {
            state.serialize_element(&ValueWithSeed::new(
                value,
                self.cast::<ValueContainer>(),
            ))?;
        }
        state.end()
    }
}

impl<'de, 'ctx> serde::de::DeserializeSeed<'de> for SerdeContext<'ctx, List> {
    type Value = List;

    fn deserialize<D>(self, deserializer: D) -> Result<List, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> serde::de::Visitor<'de> for SerdeContext<'ctx, List> {
    type Value = List;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a sequence of values for List")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<List, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut list = List::default();

        while let Some(value) =
            seq.next_element_seed(self.cast::<ValueContainer>())?
        {
            list.push(value);
        }

        Ok(list)
    }
}
