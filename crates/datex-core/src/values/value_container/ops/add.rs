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
use core::{ops::Add, result::Result};

impl Add<ValueContainer> for ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    fn add(self, rhs: ValueContainer) -> Self::Output {
        (&self).add(&rhs)
    }
}

impl Add<&ValueContainer> for &ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    // FIXME: remove clones
    fn add(self, rhs: &ValueContainer) -> Self::Output {
        match (self, rhs) {
            (ValueContainer::Local(lhs), ValueContainer::Local(rhs)) => {
                lhs + rhs
            }
            (ValueContainer::Shared(lhs), ValueContainer::Shared(rhs)) => lhs
                .with_collapsed_value_mut(|lhs| {
                    rhs.with_collapsed_value_mut(|rhs| {
                        lhs.clone() + rhs.clone()
                    })
                }),
            (ValueContainer::Local(lhs), ValueContainer::Shared(rhs)) => {
                rhs.with_collapsed_value_mut(|rhs| lhs + rhs)
            }
            (ValueContainer::Shared(lhs), ValueContainer::Local(rhs)) => {
                lhs.with_collapsed_value_mut(|lhs| lhs.clone() + rhs.clone())
            }
        }
        .map(ValueContainer::Local)
    }
}
