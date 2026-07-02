use crate::{
    shared_values::base_shared_value_container::BaseSharedValueContainer,
    traits::apply::{Apply, ApplyError},
    values::value_container::ValueContainer,
};
impl Apply for BaseSharedValueContainer {
    fn try_apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ApplyError> {
        self.with_collapsed_value(|value| value.try_apply(args))
    }

    fn try_apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        self.with_collapsed_value(|value| value.try_apply_single(arg))
    }
}
