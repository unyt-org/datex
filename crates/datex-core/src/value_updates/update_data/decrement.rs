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
pub struct DecrementUpdateData {
    pub value: ValueContainer,
}
impl DecrementUpdateData {
    pub fn new(value: ValueContainer) -> Self {
        DecrementUpdateData { value }
    }
}

impl<'ctx> SerdeContext<'ctx, DecrementUpdateData> {
    pub fn serialize_fields<S: SerializeSeq>(
        &mut self,
        value: &DecrementUpdateData,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&ValueWithSeed::new(
            &value.value,
            self.cast::<ValueContainer>(),
        ))?;
        Ok(())
    }
}
impl<'de> Visitor<'de> for SerdeContext<'_, DecrementUpdateData> {
    type Value = DecrementUpdateData;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "a delete entry update data sequence with 1 element (key)"
        )
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        mut self,
        mut seq: A,
    ) -> Result<Self::Value, A::Error> {
        let key = seq
            .next_element_seed(self.cast::<ValueContainer>())?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        Ok(DecrementUpdateData { value: key })
    }
}
