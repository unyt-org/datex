use crate::{
    dif::{
        update::{DIFKey, DIFUpdateData},
        value::DIFValueContainer,
    },
    shared_values::{
        observers::TransceiverId,
        shared_container::{AccessError, SharedContainer},
    },
    runtime::memory::Memory,
    values::{
        core_value::CoreValue,
        value_container::{ValueContainer, ValueKey},
    },
};
use core::{cell::RefCell, ops::FnOnce, prelude::rust_2024::*};

use crate::prelude::*;
pub enum DIFUpdateDataOrMemory<'a> {
    Update(&'a DIFUpdateData),
    Memory(&'a RefCell<Memory>),
}

impl<'a> From<&'a DIFUpdateData> for DIFUpdateDataOrMemory<'a> {
    fn from(update: &'a DIFUpdateData) -> Self {
        DIFUpdateDataOrMemory::Update(update)
    }
}

impl<'a> From<&'a RefCell<Memory>> for DIFUpdateDataOrMemory<'a> {
    fn from(memory: &'a RefCell<Memory>) -> Self {
        DIFUpdateDataOrMemory::Memory(memory)
    }
}

impl SharedContainer {
    /// Internal function that handles updates
    /// - Checks if the reference is mutable
    /// - Calls the provided handler to perform the update and get the DIFUpdateData
    /// - Notifies observers with the update data
    /// - Returns any AccessError encountered
    fn handle_update<'a>(
        &self,
        _source_id: TransceiverId,
        handler: impl FnOnce() -> Result<&'a DIFUpdateData, AccessError>,
    ) -> Result<(), AccessError> {
        if !self.is_mutable() {
            return Err(AccessError::ImmutableReference);
        }
        let _update_data = handler()?;
        // self.notify_observers(update_data.with_source(source_id));
        Ok(())
    }

    fn assert_mutable(&self) -> Result<(), AccessError> {
        if !self.is_mutable() {
            return Err(AccessError::ImmutableReference);
        }
        Ok(())
    }

    /// Sets a property on the value if applicable (e.g. for maps)
    pub fn try_set_property<'a>(
        &self,
        source_id: TransceiverId,
        maybe_dif_update_data: Option<&DIFUpdateData>,
        key: impl Into<ValueKey<'a>>,
        val: ValueContainer,
    ) -> Result<(), AccessError> {
        self.assert_mutable()?;

        let key = key.into();

        let dif_update = match maybe_dif_update_data {
            Some(update) => update,
            None => &DIFUpdateData::set(
                DIFKey::from_value_key(&key),
                DIFValueContainer::from_value_container(&val),
            ),
        };

        self.with_value_unchecked(|value| {
            value.try_set_property(key, val.clone())
        })?;

        self.notify_observers(&dif_update.with_source(source_id));
        Ok(())
    }

    /// Sets a value on the reference if it is mutable and the type is compatible.
    pub fn try_replace(
        &self,
        source_id: TransceiverId,
        maybe_dif_update_data: Option<&DIFUpdateData>,
        value: impl Into<ValueContainer>,
    ) -> Result<(), AccessError> {
        self.assert_mutable()?;

        // TODO #306: ensure type compatibility with allowed_type
        let value_container = &value.into();

        let dif_update = match maybe_dif_update_data {
            Some(update) => update,
            None => &DIFUpdateData::replace(
                DIFValueContainer::from_value_container(
                    value_container,
                ),
            ),
        };

        self.with_value_unchecked(|core_value| {
            // Set the value directly, ensuring it is a ValueContainer
            core_value.inner =
                value_container.to_value().borrow().inner.clone();
        });

        self.notify_observers(&dif_update.with_source(source_id));
        Ok(())
    }

    /// Pushes a value to the reference if it is a list.
    pub fn try_append_value<'a>(
        &self,
        source_id: TransceiverId,
        maybe_dif_update_data: Option<&DIFUpdateData>,
        value: impl Into<ValueContainer>,
    ) -> Result<(), AccessError> {
        self.assert_mutable()?;
        let value_container = value.into();

        let dif_update = match maybe_dif_update_data {
            Some(update) => update,
            None => {
                &DIFUpdateData::append(DIFValueContainer::from_value_container(
                    &value_container,
                ))
            }
        };

        self.with_value_unchecked(move |core_value| {
            match &mut core_value.inner {
                CoreValue::List(list) => {
                    list.push(value_container);
                }
                _ => {
                    return Err(AccessError::InvalidOperation(format!(
                        "Cannot push value to non-list value: {:?}",
                        core_value
                    )));
                }
            }

            Ok(())
        })?;

        self.notify_observers(&dif_update.with_source(source_id));
        Ok(())
    }

    /// Tries to delete a property from the reference if it is a map.
    /// Notifies observers if successful.
    pub fn try_delete_property<'a>(
        &self,
        source_id: TransceiverId,
        maybe_dif_update_data: Option<&DIFUpdateData>,
        key: impl Into<ValueKey<'a>>,
    ) -> Result<(), AccessError> {
        self.assert_mutable()?;
        let key = key.into();

        let dif_update = match maybe_dif_update_data {
            Some(update) => update,
            None => {
                &DIFUpdateData::delete(DIFKey::from_value_key(&key))
            }
        };

        self.with_value_unchecked(|value| {
            match value.inner {
                CoreValue::Map(ref mut map) => {
                    key.with_value_container(|key| map.delete(key))?;
                }
                CoreValue::List(ref mut list) => {
                    if let Some(index) = key.try_as_index() {
                        list.delete(index).map_err(|err| {
                            AccessError::IndexOutOfBounds(err)
                        })?;
                    } else {
                        return Err(AccessError::InvalidIndexKey);
                    }
                }
                _ => {
                    return Err(AccessError::InvalidOperation(format!(
                        "Cannot delete property '{:?}' on non-map value: {:?}",
                        key, value
                    )));
                }
            }

            Ok(())
        })?;

        self.notify_observers(&dif_update.with_source(source_id));
        Ok(())
    }

    pub fn try_clear(
        &self,
        source_id: TransceiverId,
    ) -> Result<(), AccessError> {
        self.assert_mutable()?;

        self.with_value_unchecked(|value| {
            match value.inner {
                CoreValue::Map(ref mut map) => {
                    map.clear()?;
                }
                CoreValue::List(ref mut list) => {
                    list.clear();
                }
                _ => {
                    return Err(AccessError::InvalidOperation(format!(
                        "Cannot clear non-list/map value: {:?}",
                        value
                    )));
                }
            }

            Ok(())
        })?;

        self.notify_observers(&DIFUpdateData::clear().with_source(source_id));
        Ok(())
    }

    pub fn try_list_splice<'a>(
        &self,
        source_id: TransceiverId,
        maybe_dif_update_data: Option<&DIFUpdateData>,
        range: core::ops::Range<u32>,
        items: Vec<ValueContainer>,
    ) -> Result<(), AccessError> {
        self.assert_mutable()?;

        let dif_update = match maybe_dif_update_data {
            Some(update) => update,
            None => {
                &DIFUpdateData::list_splice(
                    range.clone(),
                    items
                        .iter()
                        .map(|item| {
                            DIFValueContainer::from_value_container(
                                item,
                            )
                        })
                        .collect(),
                )
            }
        };

        self.with_value_unchecked(|value| {
            match value.inner {
                CoreValue::List(ref mut list) => {
                    list.splice(range, items);
                }
                _ => {
                    return Err(AccessError::InvalidOperation(format!(
                        "Cannot apply splice operation on non-list value: {:?}",
                        value
                    )));
                }
            }

            Ok(())
        })?;

        self.notify_observers(&dif_update.with_source(source_id));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        shared_values::shared_container::{
            AccessError, IndexOutOfBoundsError, SharedContainer, SharedContainerMutability,
        },
        runtime::memory::Memory,
        values::{
            core_values::{list::List, map::Map},
            value_container::ValueContainer,
        },
    };
    use core::{assert_matches, cell::RefCell};
    use crate::shared_values::pointer::Pointer;

    #[test]
    fn push() {
        let list = vec![
            ValueContainer::from(1),
            ValueContainer::from(2),
            ValueContainer::from(3),
        ];
        let list_ref =
            SharedContainer::try_new_mut(List::from(list).into(), Pointer::NULL).unwrap();
        list_ref
            .try_append_value(0, None, ValueContainer::from(4))
            .expect("Failed to push value to list");
        let updated_value = list_ref.try_get_property(3).unwrap();
        assert_eq!(updated_value, ValueContainer::from(4));

        // Try to push to immutable value
        let int_ref =
            SharedContainer::new(List::from(vec![ValueContainer::from(42)]), Pointer::NULL);
        let result =
            int_ref.try_append_value(0, None, ValueContainer::from(99));
        assert_matches!(result, Err(AccessError::ImmutableReference));

        // Try to push to non-list value
        let int_ref = SharedContainer::try_new_mut(42.into(), Pointer::NULL).unwrap();
        let result =
            int_ref.try_append_value(0, None, ValueContainer::from(99));
        assert_matches!(result, Err(AccessError::InvalidOperation(_)));
    }

    #[test]
    fn property() {
        let map = Map::from(vec![
            ("key1".to_string(), ValueContainer::from(1)),
            ("key2".to_string(), ValueContainer::from(2)),
        ]);
        let map_ref =
            SharedContainer::try_new_mut(ValueContainer::from(map), Pointer::NULL).unwrap();
        // Set existing property
        map_ref
            .try_set_property(0, None, "key1", ValueContainer::from(42))
            .expect("Failed to set existing property");
        let updated_value = map_ref.try_get_property("key1").unwrap();
        assert_eq!(updated_value, 42.into());

        // Set new property
        let result = map_ref.try_set_property(
            0,
            None,
            "new",
            ValueContainer::from(99),
        );
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
        let list_ref =
            SharedContainer::try_new_mut(ValueContainer::from(list), Pointer::NULL).unwrap();

        // Set existing index
        list_ref
            .try_set_property(0, None, 1, ValueContainer::from(42))
            .expect("Failed to set existing index");
        let updated_value = list_ref.try_get_property(1).unwrap();
        assert_eq!(updated_value, ValueContainer::from(42));

        // Try to set out-of-bounds index
        let result =
            list_ref.try_set_property(0, None, 5, ValueContainer::from(99));
        assert_matches!(
            result,
            Err(AccessError::IndexOutOfBounds(IndexOutOfBoundsError {
                index: 5
            }))
        );

        // Try to set index on non-map value
        let int_ref = SharedContainer::try_new_mut(42.into(), Pointer::NULL).unwrap();
        let result =
            int_ref.try_set_property(0, None, 0, ValueContainer::from(99));
        assert_matches!(result, Err(AccessError::InvalidOperation(_)));
    }

    #[test]
    fn text_property() {
        let struct_val = Map::from(vec![
            (ValueContainer::from("name"), ValueContainer::from("Alice")),
            (ValueContainer::from("age"), ValueContainer::from(30)),
        ]);
        let struct_ref =
            SharedContainer::try_new_mut(ValueContainer::from(struct_val), Pointer::NULL).unwrap();

        // Set existing property
        struct_ref
            .try_set_property(0, None, "name", ValueContainer::from("Bob"))
            .expect("Failed to set existing property");
        let name = struct_ref.try_get_property("name").unwrap();
        assert_eq!(name, "Bob".into());

        // Try to set non-existing property
        let result = struct_ref.try_set_property(
            0,
            None,
            "nonexistent",
            ValueContainer::from("Value"),
        );
        assert_matches!(result, Ok(()));

        // // Try to set property on non-struct value
        let int_ref = SharedContainer::try_new_mut(42.into(), Pointer::NULL).unwrap();
        let result = int_ref.try_set_property(
            0,
            None,
            "name",
            ValueContainer::from("Bob"),
        );
        assert_matches!(result, Err(AccessError::InvalidOperation(_)));
    }

    #[test]
    fn immutable_reference_fails() {
        let r = SharedContainer::new(42, Pointer::NULL);
        assert_matches!(
            r.try_replace(0, None, 43),
            Err(AccessError::ImmutableReference)
        );

        let r = SharedContainer::try_new_from_value_container(
            42.into(),
            None,
            Pointer::NULL,
            SharedContainerMutability::Immutable,
        )
        .unwrap();
        assert_matches!(
            r.try_replace(0, None, 43),
            Err(AccessError::ImmutableReference)
        );
    }
}
