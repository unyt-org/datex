use serde::{ser::SerializeSeq, Serializer};
use serde::ser::{SerializeMap, SerializeTuple};
use crate::dif::serde_context::SerdeContext;
use crate::types::r#type::Type;
use crate::types::type_definition::list::ListTypeDefinition;
use crate::types::type_definition::map::MapTypeDefinition;
use crate::types::type_definition::range::RangeTypeDefinition;
use crate::utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, RangeTypeDefinition> {
    type Value = RangeTypeDefinition;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
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
