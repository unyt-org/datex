use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::range::RangeTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use serde::{
    Serializer,
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, RangeTypeDefinition> {
    type Value = RangeTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&ValueWithSeed::new(
            &*value.start,
            self.cast::<Type>(),
        ))?;
        tuple.serialize_element(&ValueWithSeed::new(
            &*value.end,
            self.cast::<Type>(),
        ))?;
        tuple.end()
    }
}
