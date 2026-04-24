use crate::{
    prelude::*,
    runtime::{execution::ExecutionError, memory::Memory},
    shared_values::{
        SharedContainerMutability,
        base_shared_value_container::BaseSharedValueContainer,
        errors::{AccessError, SharedValueCreationError},
        observers::{Observer, ObserverId, TransceiverId},
    },
    traits::{apply::Apply, value_eq::ValueEq},
    types::r#type::Type,
    utils::freemap::{FreeHashMap, NextKey},
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData,
        },
        update_handler::UpdateHandler,
    },
    values::{value::Value, value_container::ValueContainer},
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
