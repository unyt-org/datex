use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    serde::{
        deserializer::{DatexDeserializer, from_value_container},
        error::DeserializationError,
    },
    shared_values::observers::TransceiverId,
    traits::{apply::Apply, value_eq::ValueEq},
    types::type_definition::TypeDefinition,
    values::{core_value::CoreValue, value_container::ValueContainer},
};
impl Apply for ValueContainer {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match self {
            ValueContainer::Local(value) => value.apply(args),
            ValueContainer::Shared(reference) => reference.apply(args),
        }
    }

    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match self {
            ValueContainer::Local(value) => value.apply_single(arg),
            ValueContainer::Shared(reference) => reference.apply_single(arg),
        }
    }
}
