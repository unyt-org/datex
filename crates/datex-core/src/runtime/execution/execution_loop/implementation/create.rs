//! This module contains the implementation of functions that create new [ValueContainer]s
use crate::{
    prelude::*,
    runtime::{
        execution::ExecutionError,
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::{
        OwnedSharedContainer, SelfOwnedSharedContainer, SharedContainer,
        SharedContainerMutability,
        base_shared_value_container::BaseSharedValueContainer,
    },
    values::value_container::ValueContainer,
};
use crate::values::value::value_classification::{ValueClassification, ValueTag};

/// Creates a new [ValueContainer] with a tagged type definition based on the provided [ValueContainer] and tag.
/// It expects the input [ValueContainer] to be a local value; otherwise, it returns an [ExecutionError::ExpectedLocalValue].
pub fn create_tagged_value_container(
    value_container: ValueContainer,
    tag: String,
) -> Result<ValueContainer, ExecutionError> {
    match value_container {
        ValueContainer::Local(mut value) => {
            // add tag type to the value
            value.classification = ValueClassification::Tag(ValueTag { tag, is_empty: false });
            Ok(ValueContainer::Local(value))
        }
        _ => Err(ExecutionError::ExpectedLocalValue),
    }
}

/// Creates a new owned shared container with the specified value, mutability, and pointer address provider.
/// The function returns a [ValueContainer] that wraps the newly created owned shared container.
pub fn create_owned_shared_container(
    value: ValueContainer,
    mutability: SharedContainerMutability,
    provider: &mut SelfOwnedPointerAddressProvider,
) -> ValueContainer {
    let shared_container = SharedContainer::Owned(
        OwnedSharedContainer::new_from_self_owned_container(
            SelfOwnedSharedContainer::new(
                BaseSharedValueContainer::new_with_inferred_allowed_type(
                    value, mutability,
                ),
                provider,
            ),
        ),
    );

    ValueContainer::Shared(shared_container)
}
