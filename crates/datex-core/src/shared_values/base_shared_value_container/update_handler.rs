use crate::{
    prelude::*,
    shared_values::base_shared_value_container::{
        BaseSharedValueContainer, observers::TransceiverId,
    },
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData, Update,
        },
        update_handler::{UpdateHandler, UpdateResult},
    },
    values::value_container::{ValueContainer, value_key::ValueKey},
};

impl UpdateHandler for BaseSharedValueContainer {
    fn try_handle_update(&mut self, update: Update) -> UpdateResult {
        self.assert_can_mutate()?;
        self.value_container.try_handle_update(update)
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        runtime::cache::shared_references_cache::SharedReferencesCache,
        shared_values::{
            SharedContainerMutability,
            base_shared_value_container::{
                BaseSharedValueContainer, observers::TransceiverId,
            },
            errors::{AccessError, IndexOutOfBoundsError},
        },
        value_updates::{
            errors::UpdateError,
            update_data::{
                AppendEntryUpdateData, ReplaceUpdateData, SetEntryUpdateData,
                Update, UpdateData, UpdateOperation,
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
        let list = vec![
            ValueContainer::from(1),
            ValueContainer::from(2),
            ValueContainer::from(3),
        ];
        let mut list_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                List::from(list),
                SharedContainerMutability::Mutable,
            );
        list_ref
            .try_handle_update(Update::new(
                TransceiverId::Local,
                UpdateData::new(UpdateOperation::append_entry(
                    ValueContainer::from(4),
                )),
            ))
            .expect("Failed to push value to list");
        let updated_value = list_ref.try_get_property(3).unwrap();
        assert_eq!(updated_value, ValueContainer::from(4));

        // Try to push to immutable value
        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                List::from(vec![ValueContainer::from(42)]),
                SharedContainerMutability::Immutable,
            );
        let result = int_ref.try_handle_update(Update::new(
            TransceiverId::Local,
            UpdateData::new(UpdateOperation::append_entry(
                ValueContainer::from(99),
            )),
        ));
        assert_matches!(result, Err(UpdateError::ImmutableValue));

        // Try to push to non-list value
        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Mutable,
            );
        let result = int_ref.try_handle_update(Update::new(
            TransceiverId::Local,
            UpdateData::new(UpdateOperation::append_entry(
                ValueContainer::from(99),
            )),
        ));
        assert_matches!(result, Err(UpdateError::InvalidUpdate))
    }

    #[test]
    fn get_set_property() {
        let map = Map::from(vec![
            ("key1".to_string(), ValueContainer::from(1)),
            ("key2".to_string(), ValueContainer::from(2)),
        ]);
        let mut map_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                ValueContainer::from(map),
                SharedContainerMutability::Mutable,
            );
        // Set existing property

        map_ref
            .try_handle_update(Update::new(
                TransceiverId::Local,
                UpdateData::new(UpdateOperation::set_entry(
                    "key1".into(),
                    ValueContainer::from(42),
                )),
            ))
            .expect("Failed to set existing property");
        let updated_value = map_ref.try_get_property("key1").unwrap();
        assert_eq!(updated_value, 42.into());

        // Set new property
        let result = map_ref.try_handle_update(Update::new(
            TransceiverId::Local,
            UpdateData::new(UpdateOperation::set_entry(
                "new".into(),
                ValueContainer::from(99),
            )),
        ));
        assert!(result.is_ok());
        let new_value = map_ref.try_get_property("new").unwrap();
        assert_eq!(new_value, 99.into());
    }

    #[test]
    fn numeric_property() {
        let list = vec![
            ValueContainer::from(1),
            ValueContainer::from(2),
            ValueContainer::from(3),
        ];
        let mut list_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                List::from(list),
                SharedContainerMutability::Mutable,
            );

        // Set existing index
        list_ref
            .try_set_entry(
                vec![],
                TransceiverId::Local,
                SetEntryUpdateData {
                    key: 1.into(),
                    value: ValueContainer::from(42),
                },
            )
            .expect("Failed to set existing index");
        let updated_value = list_ref.try_get_property(1).unwrap();
        assert_eq!(updated_value, ValueContainer::from(42));

        // Try to set out-of-bounds index
        let result = list_ref.try_set_entry(
            vec![],
            TransceiverId::Local,
            SetEntryUpdateData {
                key: 5.into(),
                value: ValueContainer::from(99),
            },
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
            );
        let result = int_ref.try_set_entry(
            vec![],
            TransceiverId::Local,
            SetEntryUpdateData {
                key: 0.into(),
                value: ValueContainer::from(99),
            },
        );
        assert_matches!(result, Err(UpdateError::InvalidUpdate));
    }

    #[test]
    fn text_property() {
        let struct_val = Map::from(vec![
            (ValueContainer::from("name"), ValueContainer::from("Alice")),
            (ValueContainer::from("age"), ValueContainer::from(30)),
        ]);
        let mut struct_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                ValueContainer::from(struct_val),
                SharedContainerMutability::Mutable,
            );

        // Set existing property
        struct_ref
            .try_set_entry(
                vec![],
                TransceiverId::Local,
                SetEntryUpdateData {
                    key: "name".into(),
                    value: ValueContainer::from("Bob"),
                },
            )
            .expect("Failed to set existing property");
        let name = struct_ref.try_get_property("name").unwrap();
        assert_eq!(name, "Bob".into());

        // Try to set non-existing property
        let result = struct_ref.try_set_entry(
            vec![],
            TransceiverId::Local,
            SetEntryUpdateData {
                key: "nonexistent".into(),
                value: ValueContainer::from("value"),
            },
        );
        assert_matches!(result, Ok(_));

        // // Try to set property on non-struct value
        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Mutable,
            );
        let result = int_ref.try_set_entry(
            vec![],
            TransceiverId::Local,
            SetEntryUpdateData {
                key: "name".into(),
                value: ValueContainer::from("Bob"),
            },
        );
        assert_matches!(result, Err(UpdateError::InvalidUpdate));
    }

    #[test]
    fn immutable_reference_fails() {
        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Immutable,
        );
        assert_matches!(
            r.try_replace(
                vec![],
                TransceiverId::Local,
                ReplaceUpdateData {
                    value: ValueContainer::from(43),
                },
            ),
            Err(UpdateError::ImmutableValue)
        );

        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Immutable,
        );
        assert_matches!(
            r.try_replace(
                vec![],
                TransceiverId::Local,
                ReplaceUpdateData {
                    value: ValueContainer::from(43),
                },
            ),
            Err(UpdateError::ImmutableValue)
        );
    }
}
