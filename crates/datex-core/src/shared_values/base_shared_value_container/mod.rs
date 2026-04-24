use crate::{
    shared_values::observers::{Observer, ObserverId},
    traits::value_eq::ValueEq,
    utils::freemap::{FreeHashMap, NextKey},
    values::{value::Value, value_container::ValueContainer},
};

use crate::{
    prelude::*,
    runtime::{execution::ExecutionError, memory::Memory},
    shared_values::{
        SharedContainerMutability,
        errors::{AccessError, SharedValueCreationError},
        observers::TransceiverId,
    },
    traits::apply::Apply,
    types::r#type::Type,
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData,
        },
        update_handler::UpdateHandler,
    },
    values::value_container::BorrowedValueKey,
};
use core::{
    fmt::{Debug, Display},
    ops::Deref,
    prelude::rust_2024::*,
};

pub struct BaseSharedValueContainer {
    /// The value of the container
    pub value_container: ValueContainer,
    /// The [Type] that is allowed to be assigned to the shared container. This is used for type checking when assigning a new value container to the shared container.
    pub allowed_type: Type,
    /// List of observer callbacks
    /// TODO: move observers to ValueContainer?
    pub observers: FreeHashMap<ObserverId, Observer>,
    pub mutability: SharedContainerMutability,
}

impl BaseSharedValueContainer {
    /// Tries to create a new [BaseSharedValueContainer] with an initial [ValueContainer],
    /// an allowed type and a [SharedContainerMutability].
    /// If the allowed [TypeDefinition] is not a superset of the [ValueContainer]'s allowed type,
    /// an error is returned
    pub fn try_new(
        value_container: ValueContainer,
        allowed_type: Type,
        mutability: SharedContainerMutability,
    ) -> Result<Self, SharedValueCreationError> {
        // TODO #286: make sure allowed type is superset of reference's allowed type

        Ok(BaseSharedValueContainer {
            value_container,
            allowed_type,
            observers: FreeHashMap::new(),
            mutability,
        })
    }

    /// Creates a new [BaseSharedValueContainer] with an initial [ValueContainer] and
    /// a [SharedContainerMutability].
    /// The allowed type is inferred from the value_container's allowed type.
    pub fn new_with_inferred_allowed_type<T: Into<ValueContainer>>(
        value_container: T,
        mutability: SharedContainerMutability,
        memory: &Memory,
    ) -> Self {
        let value_container = value_container.into();
        let allowed_type = value_container.allowed_type(memory);
        BaseSharedValueContainer {
            value_container,
            allowed_type,
            observers: FreeHashMap::new(),
            mutability,
        }
    }

    /// Calls the provided callback with a mut reference to the recursively collapsed inner value of the shared container
    pub fn with_collapsed_value_mut<R>(
        &mut self,
        f: impl FnOnce(&mut Value) -> R,
    ) -> R {
        match &mut self.value_container {
            ValueContainer::Local(v) => f(v),
            ValueContainer::Shared(shared) => {
                shared.with_collapsed_value_mut(f)
            }
        }
    }

    /// Calls the provided callback with a reference to the recursively collapsed inner value of the shared container
    pub fn with_collapsed_value<R>(&self, f: impl FnOnce(&Value) -> R) -> R {
        match &self.value_container {
            ValueContainer::Local(v) => f(v),
            ValueContainer::Shared(shared) => shared.with_collapsed_value(f),
        }
    }

    /// Sets the currently assigned [ValueContainer] of the shared container to a new value container.
    /// Returns the [ValueContainer] as an error if the new value container's allowed type is not compatible with the allowed type of the shared container
    pub fn try_set_value_container(
        &mut self,
        new_value_container: ValueContainer,
    ) -> Result<(), ValueContainer> {
        // TODO do type checking to ensure new value container's allowed type is compatible with self.allowed_type
        self.value_container = new_value_container;
        Ok(())
    }

    pub fn try_get_property<'a>(
        &self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, AccessError> {
        self.with_collapsed_value(|value| value.try_get_property(key))
    }
}

impl Debug for BaseSharedValueContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReferenceData")
            .field("value_container", &self.value_container)
            .field("allowed_type", &self.allowed_type)
            .field("observers", &self.observers.len())
            .finish()
    }
}

impl Display for BaseSharedValueContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "shared {}{}",
            self.value_container,
            match &self.mutability {
                SharedContainerMutability::Mutable => "mut ",
                SharedContainerMutability::Immutable => "",
            }
        )
    }
}

impl PartialEq for BaseSharedValueContainer {
    fn eq(&self, other: &Self) -> bool {
        // Two value references are equal if their value containers are equal
        self.value_container.value_eq(&other.value_container)
    }
}

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

impl BaseSharedValueContainer {
    pub fn current_value_container(&self) -> &ValueContainer {
        &self.value_container
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self.mutability, SharedContainerMutability::Mutable)
    }

    pub fn assert_can_mutate(&self) -> Result<(), UpdateError> {
        if !self.is_mutable() {
            return Err(UpdateError::ImmutableValue);
        }
        Ok(())
    }
}

impl UpdateHandler for BaseSharedValueContainer {
    fn try_replace(
        &mut self,
        data: ReplaceUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        self.assert_can_mutate()?;
        self.value_container.try_replace(data, source_id)
    }

    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        self.assert_can_mutate()?;

        self.value_container.try_set_entry(data, source_id)?;

        // self.notify_observers(&data.with_source(source_id));
        Ok(())
    }

    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        self.assert_can_mutate()?;
        self.value_container.try_delete_entry(data, source_id)
    }

    fn try_append_entry(
        &mut self,
        data: AppendEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        self.assert_can_mutate()?;
        self.value_container.try_append_entry(data, source_id)
    }

    fn try_clear(
        &mut self,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        self.assert_can_mutate()?;
        self.value_container.try_clear(source_id)
    }

    fn try_list_splice(
        &mut self,
        data: ListSpliceUpdateData,
        source_id: TransceiverId,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        self.assert_can_mutate()?;
        self.value_container.try_list_splice(data, source_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        runtime::memory::Memory,
        shared_values::{
            SharedContainerMutability,
            base_shared_value_container::BaseSharedValueContainer,
            errors::{AccessError, IndexOutOfBoundsError},
            observers::TransceiverId,
        },
        value_updates::{
            errors::UpdateError,
            update_data::{
                AppendEntryUpdateData, ReplaceUpdateData, SetEntryUpdateData,
            },
            update_handler::UpdateHandler,
        },
        values::{
            core_values::{list::List, map::Map},
            value_container::ValueContainer,
        },
    };
    use core::assert_matches;

    #[test]
    fn push() {
        let memory = &Memory::new();
        let list = vec![
            ValueContainer::from(1),
            ValueContainer::from(2),
            ValueContainer::from(3),
        ];
        let mut list_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                List::from(list),
                SharedContainerMutability::Mutable,
                memory,
            );
        list_ref
            .try_append_entry(
                AppendEntryUpdateData {
                    value: ValueContainer::from(4),
                },
                TransceiverId(0),
            )
            .expect("Failed to push value to list");
        let updated_value = list_ref.try_get_property(3).unwrap();
        assert_eq!(updated_value, ValueContainer::from(4));

        // Try to push to immutable value
        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                List::from(vec![ValueContainer::from(42)]),
                SharedContainerMutability::Immutable,
                memory,
            );
        let result = int_ref.try_append_entry(
            AppendEntryUpdateData {
                value: ValueContainer::from(99),
            },
            TransceiverId(0),
        );
        assert_matches!(result, Err(UpdateError::ImmutableValue));

        // Try to push to non-list value
        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Mutable,
                memory,
            );
        let result = int_ref.try_append_entry(
            AppendEntryUpdateData {
                value: ValueContainer::from(99),
            },
            TransceiverId(0),
        );
        assert_matches!(result, Err(UpdateError::InvalidUpdate))
    }

    #[test]
    fn get_set_property() {
        let memory = &Memory::new();
        let map = Map::from(vec![
            ("key1".to_string(), ValueContainer::from(1)),
            ("key2".to_string(), ValueContainer::from(2)),
        ]);
        let mut map_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                ValueContainer::from(map),
                SharedContainerMutability::Mutable,
                memory,
            );
        // Set existing property
        map_ref
            .try_set_entry(
                SetEntryUpdateData {
                    key: "key1".into(),
                    value: ValueContainer::from(42),
                },
                TransceiverId(0),
            )
            .expect("Failed to set existing property");
        let updated_value = map_ref.try_get_property("key1").unwrap();
        assert_eq!(updated_value, 42.into());

        // Set new property
        let result = map_ref.try_set_entry(
            SetEntryUpdateData {
                key: "new".into(),
                value: ValueContainer::from(99),
            },
            TransceiverId(0),
        );
        assert!(result.is_ok());
        let new_value = map_ref.try_get_property("new").unwrap();
        assert_eq!(new_value, 99.into());
    }

    #[test]
    fn numeric_property() {
        let memory = &Memory::new();
        let list = vec![
            ValueContainer::from(1),
            ValueContainer::from(2),
            ValueContainer::from(3),
        ];
        let mut list_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                List::from(list),
                SharedContainerMutability::Mutable,
                memory,
            );

        // Set existing index
        list_ref
            .try_set_entry(
                SetEntryUpdateData {
                    key: 1.into(),
                    value: ValueContainer::from(42),
                },
                TransceiverId(0),
            )
            .expect("Failed to set existing index");
        let updated_value = list_ref.try_get_property(1).unwrap();
        assert_eq!(updated_value, ValueContainer::from(42));

        // Try to set out-of-bounds index
        let result = list_ref.try_set_entry(
            SetEntryUpdateData {
                key: 5.into(),
                value: ValueContainer::from(99),
            },
            TransceiverId(0),
        );
        assert_matches!(
            result,
            Err(UpdateError::AccessError(AccessError::IndexOutOfBounds(
                IndexOutOfBoundsError { index: 5 }
            )))
        );

        // Try to set index on non-map value
        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Mutable,
                memory,
            );
        let result = int_ref.try_set_entry(
            SetEntryUpdateData {
                key: 0.into(),
                value: ValueContainer::from(99),
            },
            TransceiverId(0),
        );
        assert_matches!(result, Err(UpdateError::InvalidUpdate));
    }

    #[test]
    fn text_property() {
        let memory = &Memory::new();
        let struct_val = Map::from(vec![
            (ValueContainer::from("name"), ValueContainer::from("Alice")),
            (ValueContainer::from("age"), ValueContainer::from(30)),
        ]);
        let mut struct_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                ValueContainer::from(struct_val),
                SharedContainerMutability::Mutable,
                memory,
            );

        // Set existing property
        struct_ref
            .try_set_entry(
                SetEntryUpdateData {
                    key: "name".into(),
                    value: ValueContainer::from("Bob"),
                },
                TransceiverId(0),
            )
            .expect("Failed to set existing property");
        let name = struct_ref.try_get_property("name").unwrap();
        assert_eq!(name, "Bob".into());

        // Try to set non-existing property
        let result = struct_ref.try_set_entry(
            SetEntryUpdateData {
                key: "nonexistent".into(),
                value: ValueContainer::from("value"),
            },
            TransceiverId(0),
        );
        assert_matches!(result, Ok(()));

        // // Try to set property on non-struct value
        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Mutable,
                memory,
            );
        let result = int_ref.try_set_entry(
            SetEntryUpdateData {
                key: "name".into(),
                value: ValueContainer::from("Bob"),
            },
            TransceiverId(0),
        );
        assert_matches!(result, Err(UpdateError::InvalidUpdate));
    }

    #[test]
    fn immutable_reference_fails() {
        let memory = &Memory::new();
        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Immutable,
            memory,
        );
        assert_matches!(
            r.try_replace(
                ReplaceUpdateData {
                    value: ValueContainer::from(43),
                },
                TransceiverId(0)
            ),
            Err(UpdateError::ImmutableValue)
        );

        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Immutable,
            memory,
        );
        assert_matches!(
            r.try_replace(
                ReplaceUpdateData {
                    value: ValueContainer::from(43),
                },
                TransceiverId(0)
            ),
            Err(UpdateError::ImmutableValue)
        );
    }
}
