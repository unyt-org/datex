use serde::de::DeserializeSeed;
use serde::{Deserializer, Serializer};
use serde::ser::{SerializeSeq, SerializeStruct};
use crate::dif::serde_context::SerdeContext;
use crate::utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed};
use crate::value_updates::update_data::{AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData, ReplaceUpdateData, SetEntryUpdateData, Update, UpdateData, UpdateReturn};
use crate::values::value_container::value_key::ValueKey;
use crate::values::value_container::ValueContainer;

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Update> {
    type Value = Update;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        // serialize as map with {source: <TransceiverId>, data: <UpdateData>}
        let mut state = serializer.serialize_struct("Update", 2)?;
        state.serialize_field("source", &value.source_id)?;
        state.serialize_field("data", &ValueWithSeed::new(&value.data, self.cast::<UpdateData>()))?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, UpdateData> {
    type Value = UpdateData;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        match value {
            UpdateData::Replace(replace_data) => {
                self.cast::<ReplaceUpdateData>().serialize(replace_data, serializer)
            }
            UpdateData::SetEntry(set_entry_data) => {
                self.cast::<SetEntryUpdateData>().serialize(set_entry_data, serializer)
            }
            UpdateData::DeleteEntry(delete_entry_data) => {
                self.cast::<DeleteEntryUpdateData>().serialize(delete_entry_data, serializer)
            }
            UpdateData::Clear => {
                serializer.serialize_struct("Clear", 0)?.end()
            }
            UpdateData::AppendEntry(append_entry_data) => {
                self.cast::<AppendEntryUpdateData>().serialize(append_entry_data, serializer)
            }
            UpdateData::ListSplice(list_splice_data) => {
                self.cast::<ListSpliceUpdateData>().serialize(list_splice_data, serializer)
            }
        }
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, UpdateReturn> {
    type Value = UpdateReturn;
    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        match value {
            UpdateReturn::SingleValue(value) => {
                let mut state = serializer.serialize_struct("SingleValue", 1)?;
                state.serialize_field("value", &ValueWithSeed::new(value, self.cast::<ValueContainer>()))?;
                state.end()
            }
            UpdateReturn::MultipleValues(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(&ValueWithSeed::new(value, self.cast::<ValueContainer>()))?;
                }
                seq.end()
            }
            UpdateReturn::None => serializer.serialize_struct("None", 0)?.end(),
        }
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, ReplaceUpdateData> {
    type Value = ReplaceUpdateData;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ReplaceUpdateData", 1)?;
        state.serialize_field("value", &ValueWithSeed::new(&value.value, self.cast::<ValueContainer>()))?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, SetEntryUpdateData> {
    type Value = SetEntryUpdateData;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("SetEntryUpdateData", 2)?;
        state.serialize_field("key", &ValueWithSeed::new(&value.key, self.cast::<ValueKey>()))?;
        state.serialize_field("value", &ValueWithSeed::new(&value.value, self.cast::<ValueContainer>()))?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, DeleteEntryUpdateData> {
    type Value = DeleteEntryUpdateData;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("DeleteEntryUpdateData", 1)?;
        state.serialize_field("key", &ValueWithSeed::new(&value.key, self.cast::<ValueKey>()))?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, AppendEntryUpdateData> {
    type Value = AppendEntryUpdateData;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppendEntryUpdateData", 1)?;
        state.serialize_field("value", &ValueWithSeed::new(&value.value, self.cast::<ValueContainer>()))?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, ListSpliceUpdateData> {
    type Value = ListSpliceUpdateData;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ListSpliceUpdateData", 3)?;
        state.serialize_field("start", &value.start)?;
        state.serialize_field("delete_count", &value.delete_count)?;
        state.serialize_field("items", &ValueWithSeed::new(&value.items, self.cast::<Vec<ValueContainer>>()))?;
        state.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Vec<ValueContainer>> {
    type Value = Vec<ValueContainer>;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for item in value {
            seq.serialize_element(&ValueWithSeed::new(item, self.cast::<ValueContainer>()))?;
        }
        seq.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, UpdateData> {
    type Value = UpdateData;
    fn deserialize<D: Deserializer<'de>>(
        self,
        _d: D,
    ) -> Result<UpdateData, D::Error> {
        todo!()
    }
}


/// Deserialization for [ReplaceUpdateData] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, ReplaceUpdateData> {
    type Value = ReplaceUpdateData;
    fn deserialize<D: Deserializer<'de>>(
        mut self,
        _d: D,
    ) -> Result<ReplaceUpdateData, D::Error> {
        // deserialize value container
        let value = self.cast::<ValueContainer>().deserialize(_d)?;
        Ok(ReplaceUpdateData { value })
    }
}

/// Deserialization for [SetEntryUpdateData] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, SetEntryUpdateData> {
    type Value = SetEntryUpdateData;
    fn deserialize<D: Deserializer<'de>>(
        mut self,
        _d: D,
    ) -> Result<SetEntryUpdateData, D::Error> {
        todo!()
    }
}