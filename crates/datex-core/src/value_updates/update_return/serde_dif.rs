use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
    value_updates::UpdateReturn,
    values::value_container::ValueContainer,
};

use serde::{
    Serializer,
    ser::{SerializeSeq, SerializeStruct},
};

impl<'ctx> SerializeSeed for UpdateReturn {
    fn serialize_seed<S: Serializer>(
        &mut self,
        ctx: &SerdeContext<'ctx>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(None)?;

        match self {
            UpdateReturn::None => {
                seq.serialize_element("none")?;
            }
            UpdateReturn::SingleValue(value) => {
                seq.serialize_element("single_value")?;
                seq.serialize_element(&ValueWithSerdeContext::new(
                    value,
                    ctx,
                ))?;
            }
            UpdateReturn::MultipleValues(values) => {
                seq.serialize_element("multiple_values")?;
                seq.serialize_element(&ValueWithSerdeContext::new(
                    values,
                    ctx,
                ))?;
            }
        }
        seq.end()
    }
}
