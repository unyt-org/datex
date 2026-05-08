use serde::{Serialize, Serializer, ser::SerializeStruct};

use crate::values::core_values::range::Range;

impl Serialize for Range {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Range", 2)?;

        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;

        state.end()
    }
}
