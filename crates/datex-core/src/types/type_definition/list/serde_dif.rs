use serde::{ser::SerializeSeq, Serializer};
use crate::dif::serde_context::SerdeContext;
use crate::types::r#type::Type;
use crate::types::type_definition::list::ListTypeDefinition;
use crate::utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, ListTypeDefinition> {
    type Value = ListTypeDefinition;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for item in value.iter() {
            seq.serialize_element(&ValueWithSeed::new(
                item,
                self.cast::<Type>(),
            ))?;
        }
        seq.end()
    }
}
