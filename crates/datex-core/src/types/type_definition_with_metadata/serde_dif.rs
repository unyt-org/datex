use crate::{
    dif::serde_context::SerdeContext,
    types::{
        type_definition_with_metadata::TypeDefinitionWithMetadata,
    },
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
};
use serde::{Serializer, ser::SerializeSeq};

impl<'ctx> SerializeSeed for TypeDefinitionWithMetadata {

    fn serialize_seed<S>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.metadata)?;
        seq.serialize_element(&ValueWithSerdeContext::new(
            &self.definition,
            ctx,
        ))?;
        seq.end()
    }
}
