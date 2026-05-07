use serde::de::DeserializeSeed;
use serde::ser::SerializeStruct;
use serde::Serializer;
use crate::dif::serde_context::SerdeContext;
use crate::shared_values::base_shared_value_container::BaseSharedValueContainer;
use crate::utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed};
use crate::values::value_container::ValueContainer;

impl<'ctx> SerializeSeed for SerdeContext<'ctx, BaseSharedValueContainer> {
    type Value = BaseSharedValueContainer;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        // serialize as struct
        let mut state = serializer.serialize_struct("BaseSharedValueContainer", 1)?;
        state.serialize_field("mutability", &value.mutability)?;
        // TODO:
        // state.serialize_field("allowed_type", &value.allowed_type)?;
        state.serialize_field("value", &ValueWithSeed::new(&value.value_container, self.cast::<ValueContainer>()))?;
        state.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, BaseSharedValueContainer> {
    type Value = BaseSharedValueContainer;

    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        todo!()
    }
}