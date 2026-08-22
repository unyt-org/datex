use crate::{
    traits::apply::{Apply, ApplyError},
    values::{
        core_values::callable::Callable, value_container::ValueContainer,
    },
};

impl Apply for Callable {
    fn try_apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ApplyError> {
        Ok(self.call(args)?)
    }
    fn try_apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        Ok(self.call(core::slice::from_ref(arg))?)
    }
}
