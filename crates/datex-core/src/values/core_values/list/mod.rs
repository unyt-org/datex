//! This module contains the implementation of the [List] struct, which represents a list of values in the type system.

use crate::{
    prelude::*, shared_values::errors::IndexOutOfBoundsError,
    values::value_container::ValueContainer,
};
pub mod equality;
pub mod serde_dif;
use crate::value_updates::update_handler::UpdateCallbackData;
use core::{
    fmt::Display,
    ops::{Index, Range},
    result::Result,
};

mod child_iterator;
pub mod update_handler;

#[derive(Debug, Default)]
pub struct List {
    items: Vec<ValueContainer>,
    /// Optional observer callback for local values. This is used to notify observers of changes to the value.
    pub update_callback_data: Option<UpdateCallbackData>,
}

impl Clone for List {
    fn clone(&self) -> Self {
        List {
            items: self.items.clone(),
            update_callback_data: None,
        }
    }
}

impl List {
    pub fn new<T: Into<ValueContainer>>(values: Vec<T>) -> Self {
        List {
            items: values.into_iter().map(Into::into).collect(),
            update_callback_data: None,
        }
    }
    pub fn with_capacity(capacity: u32) -> Self {
        List {
            items: Vec::with_capacity(capacity as usize),
            update_callback_data: None,
        }
    }
    pub fn len(&self) -> u32 {
        self.items.len() as u32
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn try_get(
        &self,
        index: i64,
    ) -> Result<&ValueContainer, IndexOutOfBoundsError> {
        let index = self.wrap_index(index);
        self.items
            .get(index as usize)
            .ok_or(IndexOutOfBoundsError { index })
    }

    /// Sets the value at the specified index.
    /// If the index is equal to the current length of the list, the value is pushed to the end.
    /// If the index is greater than the current length, None is returned.
    /// Returns the previous value at the index if it was replaced.
    pub fn try_set(
        &mut self,
        index: i64,
        value: ValueContainer,
    ) -> Result<ValueContainer, IndexOutOfBoundsError> {
        let index = self.get_valid_index(index)?;
        // replace
        Ok(core::mem::replace(&mut self.items[index], value))
    }

    /// Tries to delete the value at the specified index, returning it if successful.
    /// If the index is out of bounds, an error is returned.
    pub fn try_delete(
        &mut self,
        index: i64,
    ) -> Result<ValueContainer, IndexOutOfBoundsError> {
        let index = self.get_valid_index(index)?;
        Ok(self.items.remove(index))
    }

    pub fn push<T: Into<ValueContainer>>(&mut self, value: T) {
        self.items.push(value.into());
    }

    pub fn pop(&mut self) -> Option<ValueContainer> {
        self.items.pop()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn as_vec(&self) -> &Vec<ValueContainer> {
        &self.items
    }

    pub fn into_vec(self) -> Vec<ValueContainer> {
        self.items
    }

    pub fn as_mut_vec(&mut self) -> &mut Vec<ValueContainer> {
        &mut self.items
    }

    pub fn iter(&self) -> core::slice::Iter<'_, ValueContainer> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, ValueContainer> {
        self.items.iter_mut()
    }

    pub fn splice(
        &mut self,
        range: Range<u32>,
        replace_with: impl IntoIterator<Item = ValueContainer>,
    ) -> Vec<ValueContainer> {
        let range = Range {
            start: range.start as usize,
            end: range.end as usize,
        };
        self.items.splice(range, replace_with).collect()
    }

    /// if index is negative, count from the end
    #[inline]
    fn wrap_index(&self, index: i64) -> u32 {
        if index < 0 {
            (index + self.items.len() as i64) as u32
        } else {
            index as u32
        }
    }

    #[inline]
    fn get_valid_index(
        &self,
        index: i64,
    ) -> Result<usize, IndexOutOfBoundsError> {
        let index = self.wrap_index(index);
        if (index as usize) < self.items.len() {
            Ok(index as usize)
        } else {
            Err(IndexOutOfBoundsError { index })
        }
    }

    pub fn delete(
        &mut self,
        index: i64,
    ) -> Result<ValueContainer, IndexOutOfBoundsError> {
        let index = self.get_valid_index(index)?;
        Ok(self.items.remove(index))
    }
}

impl Display for List {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "[")?;
        for (i, value) in self.items.iter().enumerate() {
            if i > 0 {
                core::write!(f, ", ")?;
            }
            core::write!(f, "{value}")?;
        }
        core::write!(f, "]")
    }
}

impl<T> From<Vec<T>> for List
where
    T: Into<ValueContainer>,
{
    fn from(vec: Vec<T>) -> Self {
        List {
            items: vec.into_iter().map(Into::into).collect(),
            update_callback_data: None,
        }
    }
}

impl<T> FromIterator<T> for List
where
    T: Into<ValueContainer>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        List {
            items: iter.into_iter().map(Into::into).collect(),
            update_callback_data: None,
        }
    }
}

impl Index<usize> for List {
    type Output = ValueContainer;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl IntoIterator for List {
    type Item = ValueContainer;
    type IntoIter = vec::IntoIter<ValueContainer>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a List {
    type Item = &'a ValueContainer;
    type IntoIter = core::slice::Iter<'a, ValueContainer>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

pub macro datex_list {
    ( $( $x:expr ),* ) => {
        {
            let list = alloc::vec![$( $crate::values::value_container::ValueContainer::from($x) ),*];
            $crate::values::core_values::list::List::new(list)
        }
    }
}

impl From<List> for Vec<ValueContainer> {
    fn from(list: List) -> Self {
        list.items
    }
}
