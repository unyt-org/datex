use core::fmt::Display;

use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use serde::{Serializer, ser::SerializeSeq};

use crate::types::r#type::Type;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ListSliceCollectionTypeDefinition {
    pub item_type: Box<Type>,
    pub size: usize,
}
impl Display for ListSliceCollectionTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "[{}; {}]", self.item_type, self.size)
    }
}

impl<'ctx> SerializeSeed
    for SerdeContext<'ctx, ListSliceCollectionTypeDefinition>
{
    type Value = ListSliceCollectionTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.item_type as &Type,
            self.cast::<Type>(),
        ))?;
        seq.serialize_element(&value.size)?;
        seq.end()
    }
}
