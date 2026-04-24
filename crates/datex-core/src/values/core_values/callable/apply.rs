use crate::{
    runtime::execution::ExecutionError,
    traits::apply::Apply,
    values::{
        core_values::callable::Callable, value_container::ValueContainer,
    },
};

impl Apply for Callable {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.call(args)
    }
    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.call(&[arg.clone()])
    }
}
