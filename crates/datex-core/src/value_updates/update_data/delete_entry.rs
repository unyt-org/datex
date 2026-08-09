use crate::values::value_container::value_key::ValueKey;
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
pub struct DeleteEntryUpdateData {
    pub key: ValueKey,
}
impl DeleteEntryUpdateData {
    pub fn new(key: ValueKey) -> Self {
        DeleteEntryUpdateData { key }
    }
}

impl<'ctx> SerdeContext<'ctx, DeleteEntryUpdateData> {
    pub fn serialize_fields<S: SerializeSeq>(
        &mut self,
        value: &DeleteEntryUpdateData,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&ValueWithSeed::new(
            &value.key,
            self.cast::<ValueKey>(),
        ))?;
        Ok(())
    }
}
impl<'de> Visitor<'de> for SerdeContext<'_, DeleteEntryUpdateData> {
    type Value = DeleteEntryUpdateData;

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
            .next_element_seed(self.cast::<ValueKey>())?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        Ok(DeleteEntryUpdateData { key })
    }
}
