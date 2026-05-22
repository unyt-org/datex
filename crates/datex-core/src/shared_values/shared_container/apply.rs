use crate::{
    shared_values::SharedContainer,
    traits::apply::{Apply, ApplyError},
    values::value_container::ValueContainer,
};
impl Apply for SharedContainer {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ApplyError> {
        self.base_shared_container().apply(args)
    }

    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        self.base_shared_container().apply_single(arg)
    }
}
