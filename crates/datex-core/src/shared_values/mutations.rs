// use crate::{
//     runtime::memory::Memory,
//     shared_values::{
//         SharedContainerMutability,
//         base_shared_value_container::BaseSharedValueContainer,
//         observers::TransceiverId,
//     },
//     values::{
//         core_value::CoreValue,
//         value_container::{ValueContainer, BorrowedValueKey},
//     },
// };
// use core::{cell::RefCell, ops::FnOnce, prelude::rust_2024::*};
//
// use crate::{prelude::*, shared_values::errors::AccessError};
// use crate::value_updates::errors::UpdateError;
// use crate::value_updates::update_data::{AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData, ReplaceUpdateData, SetEntryUpdateData, UpdateData};
// use crate::value_updates::update_handler::UpdateHandler;
//
// pub enum UpdateDataOrMemory<'a> {
//     Update(&'a UpdateData),
//     Memory(&'a RefCell<Memory>),
// }
//
// impl<'a> From<&'a UpdateData> for UpdateDataOrMemory<'a> {
//     fn from(update: &'a UpdateData) -> Self {
//         UpdateDataOrMemory::Update(update)
//     }
// }
//
// impl<'a> From<&'a RefCell<Memory>> for UpdateDataOrMemory<'a> {
//     fn from(memory: &'a RefCell<Memory>) -> Self {
//         UpdateDataOrMemory::Memory(memory)
//     }
// }
//
// impl BaseSharedValueContainer {
//
//     /// Internal function that handles updates
//     /// - Checks if the reference is mutable
//     /// - Calls the provided handler to perform the update and get the DIFUpdateData
//     /// - Notifies observers with the update data
//     /// - Returns any AccessError encountered
//     fn handle_update<'a>(
//         &self,
//         _source_id: TransceiverId,
//         handler: impl FnOnce() -> Result<&'a UpdateData, AccessError>,
//     ) -> Result<(), UpdateError> {
//         if !self.can_mutate() {
//             return Err(UpdateError::ImmutableValue);
//         }
//         let _update_data = handler()?;
//         // self.notify_observers(update_data.with_source(source_id));
//         Ok(())
//     }
//
//
//     // /// Sets a property on the value if applicable (e.g. for maps)
//     // pub fn try_set_property<'a>(
//     //     &mut self,
//     //     source_id: TransceiverId,
//     //     maybe_dif_update_data: Option<&UpdateData>,
//     //     key: impl Into<BorrowedValueKey<'a>>,
//     //     val: ValueContainer,
//     // ) -> Result<(), AccessError> {
//     //     self.assert_can_mutate()?;
//     //
//     //     let key = key.into();
//     //
//     //     let dif_update = match maybe_dif_update_data {
//     //         Some(update) => update,
//     //         None => &UpdateData::set(
//     //             DIFKey::from_value_key(&key),
//     //             DIFValueContainer::from_value_container(&val),
//     //         ),
//     //     };
//     //
//     //     self.value_container.try_set_property(
//     //         source_id,
//     //         maybe_dif_update_data,
//     //         key,
//     //         val,
//     //     )?;
//     //
//     //     self.notify_observers(&dif_update.with_source(source_id));
//     //     Ok(())
//     // }
//
//     // /// Sets a value on the reference if it is mutable and the type is compatible.
//     // pub fn try_replace(
//     //     &mut self,
//     //     source_id: TransceiverId,
//     //     maybe_dif_update_data: Option<&DIFUpdateData>,
//     //     value: impl Into<ValueContainer>,
//     // ) -> Result<(), AccessError> {
//     //     self.assert_can_mutate()?;
//     //
//     //     // TODO #306: ensure type compatibility with allowed_type
//     //     let value_container = value.into();
//     //
//     //     let dif_update = match maybe_dif_update_data {
//     //         Some(update) => update,
//     //         None => &DIFUpdateData::replace(
//     //             DIFValueContainer::from_value_container(&value_container),
//     //         ),
//     //     };
//     //
//     //     self.value_container = value_container;
//     //     self.notify_observers(&dif_update.with_source(source_id));
//     //     Ok(())
//     // }
//     //
//     // /// Pushes a value to the reference if it is a list.
//     // pub fn try_append_value<'a>(
//     //     &mut self,
//     //     source_id: TransceiverId,
//     //     maybe_dif_update_data: Option<&DIFUpdateData>,
//     //     value: impl Into<ValueContainer>,
//     // ) -> Result<(), AccessError> {
//     //     self.assert_can_mutate()?;
//     //     let value_container = value.into();
//     //
//     //     let dif_update = match maybe_dif_update_data {
//     //         Some(update) => update,
//     //         None => &DIFUpdateData::append(
//     //             DIFValueContainer::from_value_container(&value_container),
//     //         ),
//     //     };
//     //
//     //     self.value_container.with_collapsed_value_mut(|value| {
//     //         match &mut value.inner {
//     //             CoreValue::List(list) => {
//     //                 list.push(value_container);
//     //             }
//     //             _ => {
//     //                 return Err(AccessError::InvalidOperation(format!(
//     //                     "Cannot push value to non-list value: {:?}",
//     //                     value
//     //                 )));
//     //             }
//     //         }
//     //         Ok(())
//     //     })?;
//     //
//     //     self.notify_observers(&dif_update.with_source(source_id));
//     //     Ok(())
//     // }
//     //
//     // /// Tries to delete a property from the reference if it is a map.
//     // /// Notifies observers if successful.
//     // pub fn try_delete_property<'a>(
//     //     &mut self,
//     //     source_id: TransceiverId,
//     //     maybe_dif_update_data: Option<&DIFUpdateData>,
//     //     key: impl Into<BorrowedValueKey<'a>>,
//     // ) -> Result<(), AccessError> {
//     //     self.assert_can_mutate()?;
//     //     let key = key.into();
//     //
//     //     let dif_update = match maybe_dif_update_data {
//     //         Some(update) => update,
//     //         None => &DIFUpdateData::delete(DIFKey::from_value_key(&key)),
//     //     };
//     //
//     //     self.value_container.try_delete_property(
//     //         source_id,
//     //         maybe_dif_update_data,
//     //         key,
//     //     )?;
//     //
//     //     self.notify_observers(&dif_update.with_source(source_id));
//     //     Ok(())
//     // }
//     //
//     // pub fn try_clear(
//     //     &mut self,
//     //     source_id: TransceiverId,
//     // ) -> Result<(), AccessError> {
//     //     self.assert_can_mutate()?;
//     //
//     //     self.value_container.with_collapsed_value_mut(|value| {
//     //         match value.inner {
//     //             CoreValue::Map(ref mut map) => {
//     //                 map.clear()?;
//     //             }
//     //             CoreValue::List(ref mut list) => {
//     //                 list.clear();
//     //             }
//     //             _ => {
//     //                 return Err(AccessError::InvalidOperation(format!(
//     //                     "Cannot clear non-list/map value: {:?}",
//     //                     value
//     //                 )));
//     //             }
//     //         }
//     //
//     //         Ok(())
//     //     })?;
//     //
//     //     self.notify_observers(&DIFUpdateData::clear().with_source(source_id));
//     //     Ok(())
//     // }
//     //
//     // pub fn try_list_splice<'a>(
//     //     &mut self,
//     //     source_id: TransceiverId,
//     //     maybe_dif_update_data: Option<&DIFUpdateData>,
//     //     range: core::ops::Range<u32>,
//     //     items: Vec<ValueContainer>,
//     // ) -> Result<(), AccessError> {
//     //     self.assert_can_mutate()?;
//     //
//     //     let dif_update = match maybe_dif_update_data {
//     //         Some(update) => update,
//     //         None => &DIFUpdateData::list_splice(
//     //             range.clone(),
//     //             items
//     //                 .iter()
//     //                 .map(DIFValueContainer::from_value_container)
//     //                 .collect(),
//     //         ),
//     //     };
//     //
//     //     self.value_container.with_collapsed_value_mut(|value| {
//     //         match value.inner {
//     //             CoreValue::List(ref mut list) => {
//     //                 list.splice(range, items);
//     //             }
//     //             _ => {
//     //                 return Err(AccessError::InvalidOperation(format!(
//     //                     "Cannot apply splice operation on non-list value: {:?}",
//     //                     value
//     //                 )));
//     //             }
//     //         }
//     //
//     //         Ok(())
//     //     })?;
//     //
//     //     self.notify_observers(&dif_update.with_source(source_id));
//     //     Ok(())
//     // }
// }
//
