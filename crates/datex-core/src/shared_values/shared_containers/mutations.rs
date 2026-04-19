use crate::{
    runtime::memory::Memory,
    shared_values::shared_containers::{
        SharedContainerMutability,
        base_shared_value_container::BaseSharedValueContainer,
        observers::TransceiverId,
    },
    values::{
        core_value::CoreValue,
        value_container::{ValueContainer, BorrowedValueKey},
    },
};
use core::{cell::RefCell, ops::FnOnce, prelude::rust_2024::*};

use crate::{prelude::*, shared_values::errors::AccessError};
use crate::value_updates::errors::UpdateError;
use crate::value_updates::update_data::{DeleteEntryUpdateData, ListSpliceUpdateData, ReplaceUpdateData, SetEntryUpdateData, UpdateData};
use crate::value_updates::update_handler::UpdateHandler;

pub enum UpdateDataOrMemory<'a> {
    Update(&'a UpdateData),
    Memory(&'a RefCell<Memory>),
}

impl<'a> From<&'a UpdateData> for UpdateDataOrMemory<'a> {
    fn from(update: &'a UpdateData) -> Self {
        UpdateDataOrMemory::Update(update)
    }
}

impl<'a> From<&'a RefCell<Memory>> for UpdateDataOrMemory<'a> {
    fn from(memory: &'a RefCell<Memory>) -> Self {
        UpdateDataOrMemory::Memory(memory)
    }
}

impl BaseSharedValueContainer {

    pub fn update(
        &mut self,
        update_data: UpdateData
    ) -> Result<(), AccessError> {
        // TODO: implement new update handling
        todo!()
    }

    /// Internal function that handles updates
    /// - Checks if the reference is mutable
    /// - Calls the provided handler to perform the update and get the DIFUpdateData
    /// - Notifies observers with the update data
    /// - Returns any AccessError encountered
    fn handle_update<'a>(
        &self,
        _source_id: TransceiverId,
        handler: impl FnOnce() -> Result<&'a UpdateData, AccessError>,
    ) -> Result<(), AccessError> {
        if !self.can_mutate() {
            return Err(AccessError::ImmutableReference);
        }
        let _update_data = handler()?;
        // self.notify_observers(update_data.with_source(source_id));
        Ok(())
    }

    /// Checks if the container is mutable
    pub fn can_mutate(&self) -> bool {
        matches!(self.mutability, SharedContainerMutability::Mutable)
    }

    fn assert_can_mutate(&self) -> Result<(), AccessError> {
        if !self.can_mutate() {
            return Err(AccessError::ImmutableReference);
        }
        Ok(())
    }

    /// Sets a property on the value if applicable (e.g. for maps)
    pub fn try_set_property<'a>(
        &mut self,
        source_id: TransceiverId,
        maybe_dif_update_data: Option<&UpdateData>,
        key: impl Into<BorrowedValueKey<'a>>,
        val: ValueContainer,
    ) -> Result<(), AccessError> {
        self.assert_can_mutate()?;

        let key = key.into();

        let dif_update = match maybe_dif_update_data {
            Some(update) => update,
            None => &UpdateData::set(
                DIFKey::from_value_key(&key),
                DIFValueContainer::from_value_container(&val),
            ),
        };

        self.value_container.try_set_property(
            source_id,
            maybe_dif_update_data,
            key,
            val,
        )?;

        self.notify_observers(&dif_update.with_source(source_id));
        Ok(())
    }

    pub fn try_get_property<'a>(
        &self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, AccessError> {
        let key = key.into();
        self.value_container.try_get_property(key)
    }

    // /// Sets a value on the reference if it is mutable and the type is compatible.
    // pub fn try_replace(
    //     &mut self,
    //     source_id: TransceiverId,
    //     maybe_dif_update_data: Option<&DIFUpdateData>,
    //     value: impl Into<ValueContainer>,
    // ) -> Result<(), AccessError> {
    //     self.assert_can_mutate()?;
    // 
    //     // TODO #306: ensure type compatibility with allowed_type
    //     let value_container = value.into();
    // 
    //     let dif_update = match maybe_dif_update_data {
    //         Some(update) => update,
    //         None => &DIFUpdateData::replace(
    //             DIFValueContainer::from_value_container(&value_container),
    //         ),
    //     };
    // 
    //     self.value_container = value_container;
    //     self.notify_observers(&dif_update.with_source(source_id));
    //     Ok(())
    // }
    // 
    // /// Pushes a value to the reference if it is a list.
    // pub fn try_append_value<'a>(
    //     &mut self,
    //     source_id: TransceiverId,
    //     maybe_dif_update_data: Option<&DIFUpdateData>,
    //     value: impl Into<ValueContainer>,
    // ) -> Result<(), AccessError> {
    //     self.assert_can_mutate()?;
    //     let value_container = value.into();
    // 
    //     let dif_update = match maybe_dif_update_data {
    //         Some(update) => update,
    //         None => &DIFUpdateData::append(
    //             DIFValueContainer::from_value_container(&value_container),
    //         ),
    //     };
    // 
    //     self.value_container.with_collapsed_value_mut(|value| {
    //         match &mut value.inner {
    //             CoreValue::List(list) => {
    //                 list.push(value_container);
    //             }
    //             _ => {
    //                 return Err(AccessError::InvalidOperation(format!(
    //                     "Cannot push value to non-list value: {:?}",
    //                     value
    //                 )));
    //             }
    //         }
    //         Ok(())
    //     })?;
    // 
    //     self.notify_observers(&dif_update.with_source(source_id));
    //     Ok(())
    // }
    // 
    // /// Tries to delete a property from the reference if it is a map.
    // /// Notifies observers if successful.
    // pub fn try_delete_property<'a>(
    //     &mut self,
    //     source_id: TransceiverId,
    //     maybe_dif_update_data: Option<&DIFUpdateData>,
    //     key: impl Into<BorrowedValueKey<'a>>,
    // ) -> Result<(), AccessError> {
    //     self.assert_can_mutate()?;
    //     let key = key.into();
    // 
    //     let dif_update = match maybe_dif_update_data {
    //         Some(update) => update,
    //         None => &DIFUpdateData::delete(DIFKey::from_value_key(&key)),
    //     };
    // 
    //     self.value_container.try_delete_property(
    //         source_id,
    //         maybe_dif_update_data,
    //         key,
    //     )?;
    // 
    //     self.notify_observers(&dif_update.with_source(source_id));
    //     Ok(())
    // }
    // 
    // pub fn try_clear(
    //     &mut self,
    //     source_id: TransceiverId,
    // ) -> Result<(), AccessError> {
    //     self.assert_can_mutate()?;
    // 
    //     self.value_container.with_collapsed_value_mut(|value| {
    //         match value.inner {
    //             CoreValue::Map(ref mut map) => {
    //                 map.clear()?;
    //             }
    //             CoreValue::List(ref mut list) => {
    //                 list.clear();
    //             }
    //             _ => {
    //                 return Err(AccessError::InvalidOperation(format!(
    //                     "Cannot clear non-list/map value: {:?}",
    //                     value
    //                 )));
    //             }
    //         }
    // 
    //         Ok(())
    //     })?;
    // 
    //     self.notify_observers(&DIFUpdateData::clear().with_source(source_id));
    //     Ok(())
    // }
    // 
    // pub fn try_list_splice<'a>(
    //     &mut self,
    //     source_id: TransceiverId,
    //     maybe_dif_update_data: Option<&DIFUpdateData>,
    //     range: core::ops::Range<u32>,
    //     items: Vec<ValueContainer>,
    // ) -> Result<(), AccessError> {
    //     self.assert_can_mutate()?;
    // 
    //     let dif_update = match maybe_dif_update_data {
    //         Some(update) => update,
    //         None => &DIFUpdateData::list_splice(
    //             range.clone(),
    //             items
    //                 .iter()
    //                 .map(DIFValueContainer::from_value_container)
    //                 .collect(),
    //         ),
    //     };
    // 
    //     self.value_container.with_collapsed_value_mut(|value| {
    //         match value.inner {
    //             CoreValue::List(ref mut list) => {
    //                 list.splice(range, items);
    //             }
    //             _ => {
    //                 return Err(AccessError::InvalidOperation(format!(
    //                     "Cannot apply splice operation on non-list value: {:?}",
    //                     value
    //                 )));
    //             }
    //         }
    // 
    //         Ok(())
    //     })?;
    // 
    //     self.notify_observers(&dif_update.with_source(source_id));
    //     Ok(())
    // }
}

impl UpdateHandler for BaseSharedValueContainer  {
    fn try_replace(&self, data: ReplaceUpdateData) -> Result<ValueContainer, UpdateError> {
        todo!()
    }

    fn try_set_entry(&self, data: SetEntryUpdateData) -> Result<(), UpdateError> {
        todo!()
    }

    fn try_delete_entry(&self, data: DeleteEntryUpdateData) -> Result<ValueContainer, UpdateError> {
        todo!()
    }

    fn try_clear(&self) -> Result<Vec<ValueContainer>, UpdateError> {
        todo!()
    }

    fn try_list_splice(&self, data: ListSpliceUpdateData) -> Result<Vec<ValueContainer>, UpdateError> {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        shared_values::{
            errors::{AccessError, IndexOutOfBoundsError},
            shared_containers::{
                SharedContainer, SharedContainerMutability,
                base_shared_value_container::BaseSharedValueContainer,
            },
        },
        values::{
            core_values::{list::List, map::Map},
            value_container::ValueContainer,
        },
    };
    use core::{assert_matches, cell::RefCell};

    #[test]
    fn push() {
        let list = vec![
            ValueContainer::from(1),
            ValueContainer::from(2),
            ValueContainer::from(3),
        ];
        let mut list_ref = BaseSharedValueContainer::new_with_inferred_allowed_type(
            List::from(list),
            SharedContainerMutability::Mutable,
        );
        list_ref
            .try_append_value(0, None, ValueContainer::from(4))
            .expect("Failed to push value to list");
        let updated_value = list_ref.try_get_property(3).unwrap();
        assert_eq!(updated_value, ValueContainer::from(4));

        // Try to push to immutable value
        let mut int_ref = BaseSharedValueContainer::new_with_inferred_allowed_type(
            List::from(vec![ValueContainer::from(42)]),
            SharedContainerMutability::Immutable,
        );
        let result =
            int_ref.try_append_value(0, None, ValueContainer::from(99));
        assert_matches!(result, Err(AccessError::ImmutableReference));

        // Try to push to non-list value
        let mut int_ref = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Mutable,
        );
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
        let mut map_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                ValueContainer::from(map),
                SharedContainerMutability::Mutable,
            );
        // Set existing property
        map_ref
            .try_set_property(0, None, "key1", ValueContainer::from(42))
            .expect("Failed to set existing property");
        let updated_value = map_ref.try_get_property("key1").unwrap();
        assert_eq!(updated_value, 42.into());

        // Set new property
        let result =
            map_ref.try_set_property(0, None, "new", ValueContainer::from(99));
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
        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Mutable,
            );
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
        let mut struct_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                ValueContainer::from(struct_val),
                SharedContainerMutability::Mutable,
            );

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
        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Mutable,
            );
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
        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Immutable,
        );
        assert_matches!(
            r.try_replace(0, None, 43),
            Err(AccessError::ImmutableReference)
        );

        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Immutable,
        );
        assert_matches!(
            r.try_replace(0, None, 43),
            Err(AccessError::ImmutableReference)
        );
    }
}
