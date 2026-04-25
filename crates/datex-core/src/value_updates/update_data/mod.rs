use crate::{
    dif::deserialization_context::DeserializationContext,
    prelude::*,
    serde::Deserialize,
    shared_values::observers::TransceiverId,
    values::value_container::{ValueContainer, value_key::ValueKey},
};
use serde::{Deserializer, Serialize, de::DeserializeSeed};

#[derive(Clone, Debug, PartialEq)]
pub enum UpdateData {
    /// Represents a replacement operation for a value.
    Replace(ReplaceUpdateData),

    /// Represents an update to a specific property of a value.
    /// The `key` specifies which property to update, and `value` is the new value for that property.
    SetEntry(SetEntryUpdateData),

    /// Represents the removal of a specific property from a value.
    DeleteEntry(DeleteEntryUpdateData),

    /// Represents clearing all elements from a collection-type value (like an array or map).
    Clear,

    /// Represents adding a new element to a collection-type value (like an array or map).
    AppendEntry(AppendEntryUpdateData),

    /// Special update operation for list values that allows splicing
    ListSplice(ListSpliceUpdateData),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReplaceUpdateData {
    pub value: ValueContainer,
}

/// Deserialization for [ReplaceUpdateData] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for DeserializationContext<'ctx, ReplaceUpdateData> {
    type Value = ReplaceUpdateData;
    fn deserialize<D: Deserializer<'de>>(
        self,
        d: D,
    ) -> Result<ReplaceUpdateData, D::Error> {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetEntryUpdateData {
    pub key: ValueKey,
    pub value: ValueContainer,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteEntryUpdateData {
    pub key: ValueKey,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppendEntryUpdateData {
    pub value: ValueContainer,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListSpliceUpdateData {
    pub start: u32,
    pub delete_count: u32,
    pub items: Vec<ValueContainer>,
}

/// Represents an update to a value from a source [TransceiverId]
#[derive(Clone, Debug, PartialEq)]
pub struct Update {
    pub source_id: TransceiverId,
    pub data: UpdateData,
}

impl Update {
    /// Creates a new [Update]
    pub fn new(source_id: TransceiverId, data: UpdateData) -> Self {
        Update { source_id, data }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UpdateReturn {
    None,
    SingleValue(ValueContainer),
    MultipleValues(Vec<ValueContainer>),
}

impl From<()> for UpdateReturn {
    fn from(_: ()) -> Self {
        UpdateReturn::None
    }
}

impl From<ValueContainer> for UpdateReturn {
    fn from(value: ValueContainer) -> Self {
        UpdateReturn::SingleValue(value)
    }
}

impl From<Vec<ValueContainer>> for UpdateReturn {
    fn from(items: Vec<ValueContainer>) -> Self {
        UpdateReturn::MultipleValues(items)
    }
}
