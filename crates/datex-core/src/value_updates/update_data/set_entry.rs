use serde::{
    de::{self, Visitor},
    ser::SerializeSeq,
};

use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::ValueWithSeed,
    values::value_container::{ValueContainer, value_key::ValueKey},
};

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct SetEntryUpdateData {
    pub key: ValueKey,
    pub value: ValueContainer,
}
impl SetEntryUpdateData {
    pub fn new(key: ValueKey, value: ValueContainer) -> Self {
        SetEntryUpdateData { key, value }
    }
}

impl<'ctx> SerdeContext<'ctx, SetEntryUpdateData> {
    pub fn serialize_fields<S: SerializeSeq>(
        &mut self,
        value: &SetEntryUpdateData,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&ValueWithSeed::new(
            &value.key,
            self.cast::<ValueKey>(),
        ))?;

        seq.serialize_element(&ValueWithSeed::new(
            &value.value,
            self.cast::<ValueContainer>(),
        ))?;

        Ok(())
    }
}
impl<'de> Visitor<'de> for SerdeContext<'_, SetEntryUpdateData> {
    type Value = SetEntryUpdateData;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "a set-entry update data sequence with 2 elements")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        mut self,
        mut seq: A,
    ) -> Result<Self::Value, A::Error> {
        let key = seq
            .next_element_seed(self.cast::<ValueKey>())?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        let value = seq
            .next_element_seed(self.cast::<ValueContainer>())?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;

        Ok(SetEntryUpdateData { key, value })
    }
}
