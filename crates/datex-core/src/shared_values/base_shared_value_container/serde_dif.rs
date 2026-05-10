use crate::{
    dif::serde_context::SerdeContext,
    shared_values::base_shared_value_container::BaseSharedValueContainer,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    values::value_container::ValueContainer,
};
use serde::{Serializer, de::DeserializeSeed, ser::SerializeStruct};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, BaseSharedValueContainer> {
    type Value = BaseSharedValueContainer;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // serialize as struct
        let mut state =
            serializer.serialize_struct("BaseSharedValueContainer", 1)?;
        state.serialize_field("mutability", &value.mutability)?;
        // TODO:
        // state.serialize_field("allowed_type", &value.allowed_type)?;
        state.serialize_field(
            "value",
            &ValueWithSeed::new(
                &value.value_container,
                self.cast::<ValueContainer>(),
            ),
        )?;
        state.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, BaseSharedValueContainer>
{
    type Value = BaseSharedValueContainer;

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        _deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        todo!()
    }
}
