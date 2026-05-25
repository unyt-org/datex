use crate::{
    dif::serde_context::SerdeContext, libs::core::type_id::CoreLibTypeId,
    types::type_definition::TypeDefinition,
    utils::serde_serialize_seed::SerializeSeed,
};
use serde::Serializer;

impl<'ctx> SerializeSeed for SerdeContext<'ctx, TypeDefinition> {
    type Value = TypeDefinition;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
        // match value {
        //     TypeDefinition::Core(core) => {
        //         self.cast::<CoreLibTypeId>().serialize(core, serializer)
        //     }
        //     _ => value.serialize(serializer),
        // }
    }
}
