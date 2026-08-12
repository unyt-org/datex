use crate::{
    prelude::*,
    runtime::Runtime,
    shared_values::base_shared_value_container::BaseSharedValueContainer,
    traits::apply::{Apply, ApplyError},
    values::value_container::ValueContainer,
};

impl Apply for BaseSharedValueContainer {
    fn try_apply_sync(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        let value = self.collapsed_value();
        value.borrow().try_apply_sync(runtime, args)
    }

    async fn try_apply_async(&self, runtime: &Runtime, args: Vec<ValueContainer>) -> Result<Option<ValueContainer>, ApplyError> {
        let value = self.collapsed_value();
        value.borrow().try_apply_async(runtime, args).await
    }
}
