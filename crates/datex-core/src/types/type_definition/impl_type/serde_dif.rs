use serde::{Serializer, ser::SerializeSeq};

use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::impl_type::ImplTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, ImplTypeDefinition> {
    type Value = ImplTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.inner_type as &Type,
            self.cast::<Type>(),
        ))?;
        seq.serialize_element(&value.impl_markers)?;
        seq.end()
    }
}
