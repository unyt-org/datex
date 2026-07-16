use crate::values::value_container::ValueContainer;
use core::fmt;

use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::ValueWithSeed,
};
use serde::{
    de::{self, Visitor},
    ser::SerializeSeq,
};
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct AppendEntryUpdateData {
    pub value: ValueContainer,
}

impl<'ctx> SerdeContext<'ctx, AppendEntryUpdateData> {
    pub fn serialize_fields<S: SerializeSeq>(
        &mut self,
        value: &AppendEntryUpdateData,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&ValueWithSeed::new(
            &value.value,
            self.cast::<ValueContainer>(),
        ))?;

        Ok(())
    }
}
impl<'de> Visitor<'de> for SerdeContext<'_, AppendEntryUpdateData> {
    type Value = AppendEntryUpdateData;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "an append entry update data sequence with 1 element")
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        mut self,
        mut seq: A,
    ) -> Result<Self::Value, A::Error> {
        let value = seq
            .next_element_seed(self.cast::<ValueContainer>())?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        Ok(AppendEntryUpdateData { value })
    }
}
