use serde::{Serialize, Serializer, ser::SerializeSeq};

use crate::values::core_values::list::List;

impl Serialize for List {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len() as usize))?;
        for item in self.iter() {
            seq.serialize_element(item)?;
        }
        seq.end()
    }
}
