use crate::{
    runtime::execution::ExecutionError,
    traits::apply::Apply,
    values::{
        core_value::CoreValue, value::Value, value_container::ValueContainer,
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
