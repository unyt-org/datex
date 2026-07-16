use core::fmt;

use crate::{
    dif::serde_context::SerdeContext, prelude::*,
    utils::serde_serialize_seed::ValueWithSeed,
    values::value_container::ValueContainer,
};
use serde::{
    de::{self, Visitor},
    ser::SerializeSeq,
};

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct ListSpliceUpdateData {
    pub start: u32,
    pub delete_count: u32,
    pub items: Vec<ValueContainer>,
}
impl<'ctx> SerdeContext<'ctx, ListSpliceUpdateData> {
    pub fn serialize_fields<S: SerializeSeq>(
        &mut self,
        value: &ListSpliceUpdateData,
        seq: &mut S,
    ) -> Result<(), S::Error> {
        seq.serialize_element(&value.start)?;
        seq.serialize_element(&value.delete_count)?;

        seq.serialize_element(&ValueWithSeed::new(
            &value.items,
            self.cast::<Vec<ValueContainer>>(),
        ))?;

        Ok(())
    }
}
impl<'de> Visitor<'de> for SerdeContext<'_, ListSpliceUpdateData> {
    type Value = ListSpliceUpdateData;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "a list splice update data sequence with 3 elements (start, delete_count, items)"
        )
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        mut self,
        mut seq: A,
    ) -> Result<Self::Value, A::Error> {
        let start = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let delete_count = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let items = seq
            .next_element_seed(self.cast::<Vec<ValueContainer>>())?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;

        Ok(ListSpliceUpdateData {
            start,
            delete_count,
            items,
        })
    }
}
