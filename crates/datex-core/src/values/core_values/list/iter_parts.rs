use core::borrow::Borrow;

use crate::{
    traits::iter_parts::IterParts,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        core_values::list::List,
    },
};

impl IterParts for List {
    fn iter_list_parts<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
        Some(Box::new(gen {
            for value in self.iter() {
                yield value.into();
            }
        }))
    }

    fn iter_list_parts_mut<'a>(
        &'a mut self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>>
    {
        Some(Box::new(gen {
            for value in self.iter_mut() {
                yield value.into();
            }
        }))
    }
}
