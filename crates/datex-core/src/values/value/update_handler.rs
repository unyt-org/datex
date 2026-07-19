use crate::{
    prelude::*,
    shared_values::base_shared_value_container::observers::TransceiverId,
    traits::local_child_path_resolver::LocalChildPathResolver,
    value_updates::{
        errors::UpdateError,
        update_data::{Update, UpdateData, UpdateOperation},
        update_handler::{
            UpdateCallbackData, UpdateHandler, UpdateHandlerImpl, UpdateResult,
        },
    },
    values::{
        core_value::CoreValue,
        value::Value,
        value_container::{ValueContainer, value_key::ValueKey},
    },
};

impl Value {
    pub(crate) fn try_update_collapsed_local_inner(
        &mut self,
        operation: UpdateOperation,
        path: Vec<ValueKey>,
        source_id: Option<TransceiverId>,
    ) -> UpdateResult {
        // first collapse path to most inner nested value
        let inner_local =
            if let Some(([first], rest)) = path.split_at_checked(1) {
                self.resolve_value_for_path(first, rest)?
            } else {
                self
            };

        // then apply update
        inner_local.try_update(operation, source_id)
    }
}

impl UpdateHandlerImpl for Value {
    fn get_update_callback_data(&self) -> Option<&UpdateCallbackData> {
        match &self.inner {
            CoreValue::Map(map) => map.get_update_callback_data(),
            CoreValue::List(list) => list.get_update_callback_data(),
            _ => None,
        }
    }

    /// Tries to update the value with the given operation.
    /// If a path first needs to be resolved, use [Value::try_update_collapsed_local_inner]
    fn try_update(
        &mut self,
        operation: UpdateOperation,
        source_id: Option<TransceiverId>,
    ) -> UpdateResult {
        match &mut self.inner {
            // collections
            CoreValue::Map(map) => map.try_update(operation, source_id),
            CoreValue::List(list) => list.try_update(operation, source_id),
            CoreValue::Integer(integer) => {
                integer.try_update(operation, source_id)
            }
            CoreValue::Decimal(decimal) => {
                decimal.try_update(operation, source_id)
            }
            CoreValue::TypedInteger(integer) => {
                integer.try_update(operation, source_id)
            }
            CoreValue::TypedDecimal(decimal) => {
                decimal.try_update(operation, source_id)
            }
            _ => Err(UpdateError::InvalidUpdate),
        }
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
            value::Value,
            value_container::{ValueContainer, value_key::ValueKey},
        },
    };
    use core::assert_matches;

    #[test]
    fn push() {
        // Push to list value
        let mut list = Value::from(List::from(vec![
            ValueContainer::from(1),
            ValueContainer::from(2),
            ValueContainer::from(3),
        ]));
        list.try_update_collapsed_local_inner(
            UpdateOperation::append_entry(ValueContainer::from(4)),
            vec![],
            None,
        )
        .expect("Failed to push value to list");
        let updated_value = list.try_get_property(3).unwrap();
        assert_eq!(updated_value, ValueContainer::from(4));

        // Try to push to non-list value
        let mut int = Value::from(42);
        let result = int.try_update_collapsed_local_inner(
            UpdateOperation::append_entry(ValueContainer::from(4)),
            vec![],
            None,
        );
        assert_matches!(result, Err(UpdateError::InvalidUpdate))
    }

    #[test]
    fn get_set_property() {
        let mut map = Value::from(Map::from(vec![
            ("key1".to_string(), ValueContainer::from(1)),
            ("key2".to_string(), ValueContainer::from(2)),
        ]));

        map.try_update_collapsed_local_inner(
            UpdateOperation::set_entry("key1".into(), ValueContainer::from(42)),
            vec![],
            None,
        )
        .expect("Failed to set existing property");
        let updated_value = map.try_get_property("key1").unwrap();
        assert_eq!(updated_value, 42.into());

        // Set new property
        let result = map.try_update_collapsed_local_inner(
            UpdateOperation::set_entry("new".into(), ValueContainer::from(99)),
            vec![],
            None,
        );
        assert!(result.is_ok());
        let new_value = map.try_get_property("new").unwrap();
        assert_eq!(new_value, 99.into());
    }

    #[test]
    fn numeric_property() {
        let mut list = Value::from(vec![
            ValueContainer::from(1),
            ValueContainer::from(2),
            ValueContainer::from(3),
        ]);

        // Set existing index
        list.try_update_collapsed_local_inner(
            UpdateOperation::set_entry(1.into(), ValueContainer::from(42)),
            vec![],
            None,
        )
        .expect("Failed to set existing index");
        let updated_value = list.try_get_property(1).unwrap();
        assert_eq!(updated_value, ValueContainer::from(42));

        // Try to set out-of-bounds index
        let result = list.try_update_collapsed_local_inner(
            UpdateOperation::set_entry(5.into(), ValueContainer::from(99)),
            vec![],
            None,
        );
        assert_eq!(
            result,
            Err(UpdateError::access_error(AccessError::IndexOutOfBounds(
                IndexOutOfBoundsError { index: 5 }
            )))
        );

        // Try to set index on non-map value
        let mut int_ref = Value::from(1);
        let result = int_ref.try_update_collapsed_local_inner(
            UpdateOperation::set_entry(0.into(), ValueContainer::from(42)),
            vec![],
            None,
        );
        assert_matches!(result, Err(UpdateError::InvalidUpdate));
    }

    #[test]
    fn text_property() {
        let mut struct_val = Value::from(Map::from(vec![
            (ValueContainer::from("name"), ValueContainer::from("Alice")),
            (ValueContainer::from("age"), ValueContainer::from(30)),
        ]));

        // Set existing property
        struct_val
            .try_update_collapsed_local_inner(
                UpdateOperation::set_entry(
                    "name".into(),
                    ValueContainer::from("Bob"),
                ),
                vec![],
                None,
            )
            .expect("Failed to set existing property");
        let name = struct_val.try_get_property("name").unwrap();
        assert_eq!(name, "Bob".into());

        // Try to set non-existing property
        let result = struct_val.try_update_collapsed_local_inner(
            UpdateOperation::set_entry(
                "non_existing".into(),
                ValueContainer::from("Charlie"),
            ),
            vec![],
            None,
        );
        assert_matches!(result, Ok(_));

        // // Try to set property on non-struct value
        let mut int = Value::from(1);
        let result = int.try_update_collapsed_local_inner(
            UpdateOperation::set_entry(
                "name".into(),
                ValueContainer::from("Bob"),
            ),
            vec![],
            None,
        );
        assert_matches!(result, Err(UpdateError::InvalidUpdate));
    }

    #[test]
    fn nested_map_property() {
        let mut nested_map = Value::from(Map::from(vec![(
            ValueContainer::from("outer"),
            ValueContainer::from(Map::from(vec![(
                ValueContainer::from("inner"),
                ValueContainer::from(1),
            )])),
        )]));

        // Set existing nested property
        nested_map
            .try_update_collapsed_local_inner(
                UpdateOperation::set_entry(
                    "inner".into(),
                    ValueContainer::from(42),
                ),
                vec![ValueKey::Text("outer".to_string())],
                None,
            )
            .expect("Failed to set existing nested property");
        let inner_value = nested_map
            .try_get_property("outer")
            .unwrap()
            .try_get_property("inner")
            .unwrap();
        assert_eq!(inner_value, 42.into());
    }
}
