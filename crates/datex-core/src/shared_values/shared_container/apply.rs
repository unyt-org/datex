use crate::{
    shared_values::{SharedContainer, traits::SharedContainerCommon},
    traits::apply::{Apply, ApplyError},
    values::value_container::ValueContainer,
};
use crate::runtime::Runtime;

impl Apply for SharedContainer {
    fn try_apply(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        self.base_shared_container().try_apply(runtime, args)
    }

    fn try_apply_single(
        &self,
        runtime: &Runtime,
        arg: ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        self.base_shared_container().try_apply_single(runtime, arg)
    }
}
