use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    serde::{
        deserializer::{DatexDeserializer, from_value_container},
        error::DeserializationError,
    },
    shared_values::observers::TransceiverId,
    traits::{
        apply::Apply, identity::Identity, structural_eq::StructuralEq,
        value_eq::ValueEq,
    },
    types::type_definition::TypeDefinition,
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData,
        },
        update_handler::UpdateHandler,
    },
    values::{
        core_value::CoreValue,
        value_container::{
            ValueContainer, error::ValueError, value_key::BorrowedValueKey,
        },
    },
};
use core::result::Result;

impl UpdateHandler for ValueContainer {
    fn try_replace(
        &mut self,
        data: ReplaceUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        match self {
            ValueContainer::Local(value) => value.try_replace(data, source_id),
            ValueContainer::Shared(reference) => reference
                .base_shared_container_mut()
                .try_replace(data, source_id),
        }
    }

    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        match self {
            ValueContainer::Local(value) => {
                value.try_set_entry(data, source_id)
            }
            ValueContainer::Shared(reference) => reference
                .base_shared_container_mut()
                .try_set_entry(data, source_id),
        }
    }

    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        match self {
            ValueContainer::Local(value) => {
                value.try_delete_entry(data, source_id)
            }
            ValueContainer::Shared(reference) => reference
                .base_shared_container_mut()
                .try_delete_entry(data, source_id),
        }
    }

    fn try_append_entry(
        &mut self,
        data: AppendEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        match self {
            ValueContainer::Local(value) => {
                value.try_append_entry(data, source_id)
            }
            ValueContainer::Shared(reference) => reference
                .base_shared_container_mut()
                .try_append_entry(data, source_id),
        }
    }

    fn try_clear(
        &mut self,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        match self {
            ValueContainer::Local(value) => value.try_clear(source_id),
            ValueContainer::Shared(reference) => {
                reference.base_shared_container_mut().try_clear(source_id)
            }
        }
    }

    fn try_list_splice(
        &mut self,
        data: ListSpliceUpdateData,
        source_id: TransceiverId,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        match self {
            ValueContainer::Local(value) => {
                value.try_list_splice(data, source_id)
            }
            ValueContainer::Shared(reference) => reference
                .base_shared_container_mut()
                .try_list_splice(data, source_id),
        }
    }
}
