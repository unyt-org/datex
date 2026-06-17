use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
    value_updates::UpdateReturn,
    values::value_container::ValueContainer,
};

use serde::{
    Serializer,
    ser::{SerializeSeq, SerializeStruct},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, UpdateReturn> {
    type Value = UpdateReturn;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(None)?;

        match value {
            UpdateReturn::None => {
                seq.serialize_element("none")?;
            }
            UpdateReturn::SingleValue(value) => {
                seq.serialize_element("single_value")?;
                seq.serialize_element(&ValueWithSeed::new(
                    value,
                    self.cast::<ValueContainer>(),
                ))?;
            }
            UpdateReturn::MultipleValues(values) => {
                seq.serialize_element("multiple_values")?;
                seq.serialize_element(&ValueWithSeed::new(
                    values,
                    self.cast::<Vec<ValueContainer>>(),
                ))?;
            }
        }
        seq.end()
    }
}
