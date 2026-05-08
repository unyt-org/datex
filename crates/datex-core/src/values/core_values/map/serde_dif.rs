use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::values::core_values::map::{BorrowedMapKey, Map};

impl<'a> Serialize for BorrowedMapKey<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            BorrowedMapKey::Text(s) => serializer.serialize_str(s),
            BorrowedMapKey::Value(v) => v.serialize(serializer),
        }
    }
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.size()))?;

        for (key, value) in self.iter() {
            map.serialize_entry(&key, value)?;
        }

        map.end()
    }
}
