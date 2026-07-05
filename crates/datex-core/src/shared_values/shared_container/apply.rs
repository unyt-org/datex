use crate::{
    shared_values::SharedContainer,
    traits::apply::{Apply, ApplyError},
    values::value_container::ValueContainer,
};
use crate::shared_values::shared_container_common::SharedContainerCommon;

impl Apply for SharedContainer {
    fn try_apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ApplyError> {
        self.base_shared_container().try_apply(args)
    }

    fn try_apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        self.base_shared_container().try_apply_single(arg)
    }
}
