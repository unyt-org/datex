use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    traits::{apply::Apply, structural_eq::StructuralEq, value_eq::ValueEq},
    types::type_definition::TypeDefinition,
    values::{
        core_value::CoreValue,
        core_values::{
            callable::{Callable, CallableBody, CallableSignature},
            integer::typed_integer::TypedInteger,
        },
        value::Value,
        value_container::ValueContainer,
    },
};

impl Apply for Value {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match self.inner {
            CoreValue::Callable(ref callable) => callable.apply(args),
            _ => Err(ExecutionError::InvalidApply),
        }
    }
    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match self.inner {
            CoreValue::Callable(ref callable) => callable.apply_single(arg),
            _ => Err(ExecutionError::InvalidApply),
        }
    }
}
