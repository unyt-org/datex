use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    shared_values::base_shared_value_container::observers::TransceiverId,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    value_updates::{
        UpdateReturn,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData, Update, UpdateData,
        },
    },
    values::value_container::{ValueContainer, value_key::ValueKey},
};

use core::fmt;
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, UpdateReturn> {
    type Value = UpdateReturn;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(None)?;

        match value {
            UpdateReturn::None => {
                seq.serialize_element("none")?;
            }
            UpdateReturn::SingleValue(value) => {
                seq.serialize_element("single_value")?;
                seq.serialize_element(&ValueWithSeed::new(
                    value,
                    self.cast::<ValueContainer>(),
                ))?;
            }
            UpdateReturn::MultipleValues(values) => {
                seq.serialize_element("multiple_values")?;
                seq.serialize_element(&ValueWithSeed::new(
                    values,
                    self.cast::<Vec<ValueContainer>>(),
                ))?;
            }
        }
        seq.end()
    }
}
