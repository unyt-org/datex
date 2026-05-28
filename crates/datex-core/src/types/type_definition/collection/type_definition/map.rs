use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use core::fmt::Display;
use serde::{Serializer, ser::SerializeSeq};

use crate::types::r#type::Type;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct MapCollectionTypeDefinition {
    pub key_type: Box<Type>,
    pub value_type: Box<Type>,
}
impl Display for MapCollectionTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "Map<{}, {}>", self.key_type, self.value_type)
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, MapCollectionTypeDefinition> {
    type Value = MapCollectionTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.key_type as &Type,
            self.cast::<Type>(),
        ))?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.value_type as &Type,
            self.cast::<Type>(),
        ))?;
        seq.end()
    }
}
