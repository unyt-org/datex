use serde::{
    de::{self, Visitor},
    ser::SerializeSeq,
};

use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::ValueWithSerdeContext,
    values::value_container::{ValueContainer, value_key::ValueKey},
};
use crate::dif::serde_context::DeserializeSerdeContext;

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

impl<'ctx> SerdeContext<'ctx> {
    pub fn serialize_set_entry_fields<S: SerializeSeq>(
        &self,
        value: &SetEntryUpdateData,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&ValueWithSerdeContext::new(
            &value.key,
            self,
        ))?;

        seq.serialize_element(&ValueWithSerdeContext::new(
            &value.value,
            self,
        ))?;

        Ok(())
    }
}
impl<'de> Visitor<'de> for DeserializeSerdeContext<'de, '_, SetEntryUpdateData> {
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
