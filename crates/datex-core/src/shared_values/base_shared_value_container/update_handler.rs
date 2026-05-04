use crate::{
    prelude::*,
    shared_values::base_shared_value_container::BaseSharedValueContainer,
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData,
        },
        update_handler::UpdateHandler,
    },
    values::value_container::ValueContainer,
};
use crate::shared_values::base_shared_value_container::observers::TransceiverId;
use crate::value_updates::update_data::UpdateData;

impl UpdateHandler for BaseSharedValueContainer {
    fn try_replace(
        &mut self,
        data: ReplaceUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        self.assert_can_mutate()?;
        // set new value container
        // TODO: type check?
        let prev = core::mem::replace(&mut self.value_container, data.value.clone());
        
        self.notify_observers(&UpdateData::Replace(data).with_source(source_id));
        Ok(prev)
    }

    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        self.assert_can_mutate()?;

        self.value_container.try_set_entry(data.clone(), source_id)?;

        self.notify_observers(&UpdateData::SetEntry(data).with_source(source_id));
        Ok(())
    }

    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        self.assert_can_mutate()?;
        let previous = self.value_container.try_delete_entry(data.clone(), source_id)?;
        
        self.notify_observers(&UpdateData::DeleteEntry(data).with_source(source_id));
        
        Ok(previous)
    }

    fn try_append_entry(
        &mut self,
        data: AppendEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        self.assert_can_mutate()?;
        self.value_container.try_append_entry(data.clone(), source_id)?;
        
        self.notify_observers(&UpdateData::AppendEntry(data).with_source(source_id));
        Ok(())
    }

    fn try_clear(
        &mut self,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        self.assert_can_mutate()?;
        self.value_container.try_clear(source_id)?;
        
        self.notify_observers(&UpdateData::Clear.with_source(source_id));
        Ok(())
    }

    fn try_list_splice(
        &mut self,
        data: ListSpliceUpdateData,
        source_id: TransceiverId,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        self.assert_can_mutate()?;
        let removed = self.value_container.try_list_splice(data.clone(), source_id)?;
        
        self.notify_observers(&UpdateData::ListSplice(data).with_source(source_id));
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        runtime::memory::Memory,
        shared_values::{
            base_shared_value_container::BaseSharedValueContainer,
            errors::{AccessError, IndexOutOfBoundsError},
            SharedContainerMutability,
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
    use crate::shared_values::base_shared_value_container::observers::TransceiverId;

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
        assert_eq!(
            result,
            Err(UpdateError::access_error(AccessError::IndexOutOfBounds(
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
