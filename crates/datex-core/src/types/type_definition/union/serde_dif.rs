use serde::{Serializer, ser::SerializeSeq};

use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::union::UnionTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, UnionTypeDefinition> {
    type Value = UnionTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for type_def in value.iter() {
            seq.serialize_element(&ValueWithSeed::new(
                type_def,
                self.cast::<Type>(),
            ))?;
        }
        seq.end()
    }
}
