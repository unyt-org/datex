use crate::{prelude::*, values::core_values::integer::Integer};
use serde::{Serialize, Serializer};

impl Serialize for Integer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}
