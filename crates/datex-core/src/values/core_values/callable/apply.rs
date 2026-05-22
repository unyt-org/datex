use crate::{
    traits::apply::{Apply, ApplyError},
    values::{
        core_values::callable::Callable, value_container::ValueContainer,
    },
};

impl Apply for Callable {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ApplyError> {
        Ok(self.call(args)?)
    }
    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        Ok(self.call(&[arg.clone()])?)
    }
}
