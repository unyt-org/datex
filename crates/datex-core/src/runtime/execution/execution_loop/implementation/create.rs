use crate::{
    global::protocol_structures::instruction_data::ModifySharedContainerValue,
    prelude::*,
    runtime::{
        execution::ExecutionError,
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::{
        OwnedSharedContainer, SelfOwnedSharedContainer, SharedContainer,
        SharedContainerMutability,
        base_shared_value_container::{
            BaseSharedValueContainer, observers::TransceiverId,
        },
        traits::SharedContainerCommon,
    },
    types::{
        r#type::Type,
        type_definition::{TypeDefinition, tagged_type::TaggedTypeDefinition},
    },
    value_updates::{
        errors::UpdateError, update_data::ReplaceUpdateData,
        update_handler::UpdateHandler,
    },
    values::value_container::ValueContainer,
};

/// Creates a new [ValueContainer] with a tagged type definition based on the provided [ValueContainer] and tag.
/// It expects the input [ValueContainer] to be a local value; otherwise, it returns an [ExecutionError::ExpectedLocalValue].
pub fn create_tagged_value_container(
    value_container: ValueContainer,
    tag: String,
) -> Result<ValueContainer, ExecutionError> {
    match value_container {
        ValueContainer::Local(mut value) => {
            // add tag type to the value
            value.custom_type =
                Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
                    tag,
                    ty: value.custom_type.map(Type::from).map(Box::new),
                }));
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
