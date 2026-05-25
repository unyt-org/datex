use serde::{ser::SerializeSeq, Serializer};
use serde::ser::SerializeMap;
use crate::dif::serde_context::SerdeContext;
use crate::types::r#type::Type;
use crate::types::type_definition::list::ListTypeDefinition;
use crate::types::type_definition::map::MapTypeDefinition;
use crate::utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, MapTypeDefinition> {
    type Value = MapTypeDefinition;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_map(Some(value.len()))?;
        for (key, value) in value.iter() {
            seq.serialize_key(&ValueWithSeed::new(
                key,
                self.cast::<Type>(),
            ))?;
            seq.serialize_value(&ValueWithSeed::new(
                value,
                self.cast::<Type>(),
            ))?;
        }
        seq.end()
    }
}
