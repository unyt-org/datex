mod serde_dif;

use strum::AsRefStr;

use crate::shared_values::base_shared_value_container::observers::TransceiverId;
mod append_entry;
mod delete_entry;
mod list_splice;
mod replace;
mod set_entry;

pub use append_entry::*;
pub use delete_entry::*;
pub use list_splice::*;
pub use replace::*;
pub use set_entry::*;

#[derive(Clone, Debug, PartialEq, AsRefStr)]
#[strum(serialize_all = "snake_case")]
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

impl UpdateData {
    /// Creates a new [Update] struct with the given [TransceiverId] as source id.
    pub fn with_source(self, source_id: TransceiverId) -> Update {
        Update {
            source_id,
            data: self,
        }
    }
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
