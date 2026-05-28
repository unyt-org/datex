use serde::{Serializer, ser::SerializeSeq};

use crate::{
    dif::serde_context::SerdeContext,
    types::type_definition::{
        TypeDefinition, tagged_type::TaggedTypeDefinition,
    },
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, TaggedTypeDefinition> {
    type Value = TaggedTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&value.tag)?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.ty,
            self.cast::<Option<Box<TypeDefinition>>>(),
        ))?;
        seq.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Option<Box<TypeDefinition>>> {
    type Value = Option<Box<TypeDefinition>>;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(ty) => {
                let mut seed = self.cast::<TypeDefinition>();
                seed.serialize(ty, serializer)
            }
            None => serializer.serialize_none(),
        }
    }
}
