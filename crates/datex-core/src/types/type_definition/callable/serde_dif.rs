use serde::{
    Deserializer, Serializer,
    de::DeserializeSeed,
    ser::{SerializeMap, SerializeSeq},
};

use crate::{
    dif::serde_context::SerdeContext,
    types::type_definition::callable::CallableTypeDefinition,
    utils::serde_serialize_seed::SerializeSeed,
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, CallableTypeDefinition> {
    type Value = CallableTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // FIXME
        let mut obj = serializer.serialize_map(Some(1))?;
        obj.serialize_key("kind")?;
        obj.serialize_value(&value.kind)?;
        // obj.serialize_key("parameter_types")?;
        // obj.serialize_value(&ValueWithSeed::new(
        //     &value.parameter_types,
        //     self.cast::<Vec<(Option<String>, Type)>>(),
        // ))?;
        // obj.serialize_key("rest_parameter_type")?;
        // obj.serialize_value(&ValueWithSeed::new(
        //     &value.rest_parameter_type,
        //     self.cast::<Option<(Option<String>, Box<Type>)>>(),
        // ))?;
        // obj.serialize_key("return_type")?;
        // obj.serialize_value(&ValueWithSeed::new(
        //     &value.return_type,
        //     self.cast::<Option<Box<Type>>>(),
        // ))?;
        obj.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, CallableTypeDefinition>
{
    type Value = CallableTypeDefinition;

    fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!(
            "deserialization for CallableTypeDefinition is not implemented yet"
        )
    }
}
