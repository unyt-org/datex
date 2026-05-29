use crate::{
    dif::serde_context::SerdeContext,
    types::{
        type_definition::TypeDefinition,
        type_definition_with_metadata::TypeDefinitionWithMetadata,
    },
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use serde::{
    Serializer,
    de::{DeserializeSeed, Visitor},
    ser::SerializeSeq,
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, TypeDefinitionWithMetadata> {
    type Value = TypeDefinitionWithMetadata;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&value.metadata)?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.definition,
            &mut self.cast::<TypeDefinition>(),
        ))?;
        seq.end()
    }
}