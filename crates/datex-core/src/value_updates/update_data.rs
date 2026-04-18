use serde::Serialize;
use crate::serde::Deserialize;
use crate::shared_values::shared_containers::observers::TransceiverId;
use crate::values::value_container::{ValueContainer, ValueKey};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpdateData {
    /// Represents a replacement operation for a value.
    Replace { value: ValueContainer },

    /// Represents an update to a specific property of a value.
    /// The `key` specifies which property to update, and `value` is the new value for that property.
    Set {
        key: ValueKey,
        value: ValueContainer,
    },

    /// Represents the removal of a specific property from a value.
    Delete { key: ValueKey },

    /// Represents clearing all elements from a collection-type value (like an array or map).
    Clear,

    /// Represents adding a new element to a collection-type value (like an array or map).
    Append { value: ValueContainer },

    /// Special update operation for list values that allows splicing
    ListSplice {
        start: u32,
        delete_count: u32,
        items: Vec<ValueContainer>,
    },
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