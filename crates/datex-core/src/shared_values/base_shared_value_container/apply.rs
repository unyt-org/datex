use crate::{
    runtime::execution::ExecutionError,
    shared_values::base_shared_value_container::BaseSharedValueContainer,
    traits::apply::Apply, values::value_container::ValueContainer,
};
impl Apply for BaseSharedValueContainer {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.with_collapsed_value(|value| value.apply(args))
    }

    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.with_collapsed_value(|value| value.apply_single(arg))
    }
}
