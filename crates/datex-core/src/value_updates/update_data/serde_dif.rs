use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    shared_values::base_shared_value_container::observers::TransceiverId,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    value_updates::update_data::{
        AppendEntryUpdateData, DecrementUpdateData, DeleteEntryUpdateData,
        IncrementUpdateData, ListSpliceUpdateData, ReplaceUpdateData,
        SetEntryUpdateData, Update, UpdateData, UpdateOperation,
    },
    values::value_container::value_key::ValueKey,
};
use core::fmt;
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::SerializeSeq,
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Update> {
    type Value = Update;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(None)?;
        // serialize the transceiver id and the operation name as a string
        seq.serialize_element(value.source_id())?;

        // serialize path, if the path is empty, we pack an empty vec, otherwise we pack the path as a sequence of ValueKeys
        {
            let val = value.path();
            let path_val = ValueWithSeed::new(&val, self.cast::<&[ValueKey]>());
            seq.serialize_element(&path_val)?;
        }

        // this serializes the name of the operation (e.g. "set_entry", "replace", etc.) as a string
        {
            seq.serialize_element(&&value.operation().as_ref().to_string())?;
            match value.operation() {
                UpdateOperation::SetEntry(data) => self
                    .cast::<SetEntryUpdateData>()
                    .serialize_fields(data, &mut seq)?,
                UpdateOperation::Replace(data) => self
                    .cast::<ReplaceUpdateData>()
                    .serialize_fields(data, &mut seq)?,
                UpdateOperation::DeleteEntry(data) => self
                    .cast::<DeleteEntryUpdateData>()
                    .serialize_fields(data, &mut seq)?,
                UpdateOperation::Clear => {}
                UpdateOperation::AppendEntry(data) => self
                    .cast::<AppendEntryUpdateData>()
                    .serialize_fields(data, &mut seq)?,
                UpdateOperation::ListSplice(data) => self
                    .cast::<ListSpliceUpdateData>()
                    .serialize_fields(data, &mut seq)?,
                UpdateOperation::Increment(data) => self
                    .cast::<IncrementUpdateData>()
                    .serialize_fields(data, &mut seq)?,
                UpdateOperation::Decrement(data) => self
                    .cast::<DecrementUpdateData>()
                    .serialize_fields(data, &mut seq)?,
            };
        }
        seq.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Update> {
    type Value = Update;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Update> {
    type Value = Update;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "an update-data sequence starting with a variant hint")
    }

    fn visit_seq<A: SeqAccess<'de>>(
        mut self,
        mut seq: A,
    ) -> Result<Self::Value, A::Error> {
        let transceiver_id: TransceiverId = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;

        let path = seq
            .next_element_seed(self.cast::<Vec<ValueKey>>())?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;

        let kind: String = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        let operation = match kind.as_str() {
            "replace" => UpdateOperation::Replace(Box::new(
                self.cast::<ReplaceUpdateData>().visit_seq(&mut seq)?,
            )),

            "set_entry" => UpdateOperation::SetEntry(Box::new(
                self.cast::<SetEntryUpdateData>().visit_seq(&mut seq)?,
            )),

            "delete_entry" => UpdateOperation::DeleteEntry(Box::new(
                self.cast::<DeleteEntryUpdateData>().visit_seq(&mut seq)?,
            )),

            "clear" => UpdateOperation::Clear,

            "append_entry" => UpdateOperation::AppendEntry(Box::new(
                self.cast::<AppendEntryUpdateData>().visit_seq(&mut seq)?,
            )),

            "list_splice" => UpdateOperation::ListSplice(Box::new(
                self.cast::<ListSpliceUpdateData>().visit_seq(&mut seq)?,
            )),

            other => {
                return Err(de::Error::unknown_variant(
                    other,
                    &[
                        "replace",
                        "set_entry",
                        "delete_entry",
                        "clear",
                        "append_entry",
                        "list_splice",
                    ],
                ));
            }
        };

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::custom(format!(
                "unexpected trailing value after `{kind}` update payload"
            )));
        }

        Ok(Update::new(
            transceiver_id,
            UpdateData::new_with_path(operation, path),
        ))
    }
}
