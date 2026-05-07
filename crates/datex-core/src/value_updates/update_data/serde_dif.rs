use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    shared_values::base_shared_value_container::observers::TransceiverId,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    value_updates::update_data::{
        AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
        ReplaceUpdateData, SetEntryUpdateData, Update, UpdateData,
        UpdateReturn,
    },
    values::value_container::{ValueContainer, value_key::ValueKey},
};
use core::fmt;
use serde::{
    Deserializer, Serializer, de,
    de::{DeserializeSeed, MapAccess, Visitor, value::MapAccessDeserializer},
    ser::{SerializeSeq, SerializeStruct},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Update> {
    type Value = Update;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // serialize as map with {source: <TransceiverId>, data: <UpdateData>}
        let mut state = serializer.serialize_struct("Update", 2)?;
        state.serialize_field("source", &value.source_id)?;
        state.serialize_field(
            "data",
            &ValueWithSeed::new(&value.data, self.cast::<UpdateData>()),
        )?;
        state.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Update> {
    type Value = Update;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_struct(
            "Update",
            &["source", "data"],
            UpdateVisitor { ctx: self },
        )
    }
}

struct UpdateVisitor<'ctx> {
    ctx: SerdeContext<'ctx, Update>,
}

impl<'de, 'ctx> Visitor<'de> for UpdateVisitor<'ctx> {
    type Value = Update;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a map with `source` and `data` fields")
    }

    fn visit_map<A: MapAccess<'de>>(
        mut self,
        mut map: A,
    ) -> Result<Self::Value, A::Error> {
        let mut source_id: Option<TransceiverId> = None;
        let mut data: Option<UpdateData> = None;

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "source" => {
                    source_id = Some(map.next_value()?);
                }
                "data" => {
                    // Use next_value_seed to thread your context through
                    data = Some(
                        map.next_value_seed(self.ctx.cast::<UpdateData>())?,
                    );
                }
                other => {
                    return Err(de::Error::unknown_field(
                        other,
                        &["source", "data"],
                    ));
                }
            }
        }

        Ok(Update {
            source_id: source_id
                .ok_or_else(|| de::Error::missing_field("source"))?,
            data: data.ok_or_else(|| de::Error::missing_field("data"))?,
        })
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, UpdateData> {
    type Value = UpdateData;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            UpdateData::Replace(replace_data) => self
                .cast::<ReplaceUpdateData>()
                .serialize(replace_data, serializer),
            UpdateData::SetEntry(set_entry_data) => self
                .cast::<SetEntryUpdateData>()
                .serialize(set_entry_data, serializer),
            UpdateData::DeleteEntry(delete_entry_data) => self
                .cast::<DeleteEntryUpdateData>()
                .serialize(delete_entry_data, serializer),
            UpdateData::Clear => {
                let mut state = serializer.serialize_struct("Clear", 1)?;
                state.serialize_field("kind", "clear")?;
                state.end()
            }
            UpdateData::AppendEntry(append_entry_data) => self
                .cast::<AppendEntryUpdateData>()
                .serialize(append_entry_data, serializer),
            UpdateData::ListSplice(list_splice_data) => self
                .cast::<ListSpliceUpdateData>()
                .serialize(list_splice_data, serializer),
        }
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, UpdateReturn> {
    type Value = UpdateReturn;
    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            UpdateReturn::SingleValue(value) => {
                let mut state =
                    serializer.serialize_struct("SingleValue", 1)?;
                state.serialize_field(
                    "value",
                    &ValueWithSeed::new(value, self.cast::<ValueContainer>()),
                )?;
                state.end()
            }
            UpdateReturn::MultipleValues(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(&ValueWithSeed::new(
                        value,
                        self.cast::<ValueContainer>(),
                    ))?;
                }
                seq.end()
            }
            UpdateReturn::None => serializer.serialize_struct("None", 0)?.end(),
        }
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, ReplaceUpdateData> {
    type Value = ReplaceUpdateData;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ReplaceUpdateData", 2)?;
        state.serialize_field("kind", "replace")?;
        state.serialize_field(
            "value",
            &ValueWithSeed::new(&value.value, self.cast::<ValueContainer>()),
        )?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, SetEntryUpdateData> {
    type Value = SetEntryUpdateData;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("SetEntryUpdateData", 3)?;
        state.serialize_field("kind", "set_entry")?;
        state.serialize_field(
            "key",
            &ValueWithSeed::new(&value.key, self.cast::<ValueKey>()),
        )?;
        state.serialize_field(
            "value",
            &ValueWithSeed::new(&value.value, self.cast::<ValueContainer>()),
        )?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, DeleteEntryUpdateData> {
    type Value = DeleteEntryUpdateData;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut state =
            serializer.serialize_struct("DeleteEntryUpdateData", 2)?;
        state.serialize_field("kind", "delete_entry")?;
        state.serialize_field(
            "key",
            &ValueWithSeed::new(&value.key, self.cast::<ValueKey>()),
        )?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, AppendEntryUpdateData> {
    type Value = AppendEntryUpdateData;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut state =
            serializer.serialize_struct("AppendEntryUpdateData", 2)?;
        state.serialize_field("kind", "append_entry")?;
        state.serialize_field(
            "value",
            &ValueWithSeed::new(&value.value, self.cast::<ValueContainer>()),
        )?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, ListSpliceUpdateData> {
    type Value = ListSpliceUpdateData;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut state =
            serializer.serialize_struct("ListSpliceUpdateData", 4)?;
        state.serialize_field("kind", "list_splice")?;
        state.serialize_field("start", &value.start)?;
        state.serialize_field("delete_count", &value.delete_count)?;
        state.serialize_field(
            "items",
            &ValueWithSeed::new(
                &value.items,
                self.cast::<Vec<ValueContainer>>(),
            ),
        )?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Vec<ValueContainer>> {
    type Value = Vec<ValueContainer>;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for item in value {
            seq.serialize_element(&ValueWithSeed::new(
                item,
                self.cast::<ValueContainer>(),
            ))?;
        }
        seq.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, UpdateData> {
    type Value = UpdateData;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(UpdateDataVisitor { ctx: self })
    }
}

struct UpdateDataVisitor<'ctx> {
    ctx: SerdeContext<'ctx, UpdateData>,
}

impl<'de, 'ctx> Visitor<'de> for UpdateDataVisitor<'ctx> {
    type Value = UpdateData;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UpdateData map with a `type` field")
    }

    fn visit_map<A: MapAccess<'de>>(
        mut self,
        mut map: A,
    ) -> Result<Self::Value, A::Error> {
        // Expect `type` as the first key
        match map.next_key::<&str>()? {
            Some("type") => {}
            Some(other) => {
                return Err(de::Error::custom(format!(
                    "expected `type` field first, got `{}`",
                    other
                )));
            }
            None => return Err(de::Error::missing_field("type")),
        }

        let kind = map.next_value::<String>()?;

        todo!()
    }
}
