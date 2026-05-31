use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    shared_values::base_shared_value_container::observers::TransceiverId,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    value_updates::update_data::{
        AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
        ReplaceUpdateData, SetEntryUpdateData, Update, UpdateData,
    },
    values::value_container::{ValueContainer, value_key::ValueKey},
};
use core::fmt;
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Update> {
    type Value = Update;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element(&value.source_id.0)?;
        seq.serialize_element(&&value.data.as_ref().to_string())?;
        match &value.data {
            UpdateData::SetEntry(data) => self
                .cast::<SetEntryUpdateData>()
                .serialize_fields(data, &mut seq)?,
            UpdateData::Replace(data) => self
                .cast::<ReplaceUpdateData>()
                .serialize_fields(data, &mut seq)?,
            UpdateData::DeleteEntry(data) => self
                .cast::<DeleteEntryUpdateData>()
                .serialize_fields(data, &mut seq)?,
            UpdateData::Clear => {}
            UpdateData::AppendEntry(data) => self
                .cast::<AppendEntryUpdateData>()
                .serialize_fields(data, &mut seq)?,
            UpdateData::ListSplice(data) => self
                .cast::<ListSpliceUpdateData>()
                .serialize_fields(data, &mut seq)?,
        };
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
        let transceiver_id: u32 = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let kind: String = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        let value = match kind.as_str() {
            "replace" => UpdateData::Replace(
                self.cast::<ReplaceUpdateData>().visit_seq(&mut seq)?,
            ),

            "set_entry" => UpdateData::SetEntry(
                self.cast::<SetEntryUpdateData>().visit_seq(&mut seq)?,
            ),

            "delete_entry" => UpdateData::DeleteEntry(
                self.cast::<DeleteEntryUpdateData>().visit_seq(&mut seq)?,
            ),

            "clear" => UpdateData::Clear,

            "append_entry" => UpdateData::AppendEntry(
                self.cast::<AppendEntryUpdateData>().visit_seq(&mut seq)?,
            ),

            "list_splice" => UpdateData::ListSplice(
                self.cast::<ListSpliceUpdateData>().visit_seq(&mut seq)?,
            ),

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

        Ok(Update {
            data: value,
            source_id: TransceiverId(transceiver_id),
        })
    }
}
