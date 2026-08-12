use crate::{
    prelude::*,
    runtime::Runtime,
    shared_values::{SharedContainer, traits::SharedContainerCommon},
    traits::apply::{Apply, ApplyError},
    values::value_container::ValueContainer,
};

impl Apply for SharedContainer {
    fn try_apply_sync(
        &self,
        runtime: &Runtime,
        args: Vec<ValueContainer>,
    ) -> Result<Option<ValueContainer>, ApplyError> {
        self.base_shared_container().try_apply_sync(runtime, args)
    }

    async fn try_apply_async(&self, runtime: &Runtime, args: Vec<ValueContainer>) -> Result<Option<ValueContainer>, ApplyError> {
        self.base_shared_container().try_apply_async(runtime, args).await
    }
}
