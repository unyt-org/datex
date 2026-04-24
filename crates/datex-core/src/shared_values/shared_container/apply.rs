use crate::{
    runtime::execution::ExecutionError, shared_values::SharedContainer,
    traits::apply::Apply, values::value_container::ValueContainer,
};
impl Apply for SharedContainer {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.base_shared_container().apply(args)
    }

    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.base_shared_container().apply_single(arg)
    }
}
