use serde::Serialize;
use crate::serde::Deserialize;
use crate::shared_values::shared_containers::observers::TransceiverId;
use crate::values::value_container::{ValueContainer, ValueKey};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReplaceUpdateData {
    pub value: ValueContainer,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SetEntryUpdateData {
    pub key: ValueKey,
    pub value: ValueContainer,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeleteEntryUpdateData {
    pub key: ValueKey,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AppendEntryUpdateData {
    pub value: ValueContainer,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListSpliceUpdateData {
    pub start: u32,
    pub delete_count: u32,
    pub items: Vec<ValueContainer>,
}


/// Represents an update to a value from a source [TransceiverId]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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