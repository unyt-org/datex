use crate::{
    prelude::*,
    runtime::{execution::ExecutionError, memory::Memory},
    serde::{
        deserializer::{DatexDeserializer, from_value_container},
        error::{DeserializationError, SerializationError},
        serializer::to_value_container,
    },
    shared_values::{
        SharedContainer, errors::AccessError, observers::TransceiverId,
    },
    traits::{
        apply::Apply, identity::Identity, structural_eq::StructuralEq,
        value_eq::ValueEq,
    },
    types::{
        r#type::Type,
        type_definition::TypeDefinition,
        type_definition_with_metadata::{
            TypeDefinitionWithMetadata, TypeMetadata,
        },
    },
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
use core::{ops::Neg, result::Result};

impl Neg for ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    fn neg(self) -> Self::Output {
        match self {
            ValueContainer::Local(value) => (-value).map(ValueContainer::Local),
            ValueContainer::Shared(reference) => reference
                .with_collapsed_value_mut(|value| {
                    (-value.clone()).map(ValueContainer::Local)
                }),
        }
    }
}
