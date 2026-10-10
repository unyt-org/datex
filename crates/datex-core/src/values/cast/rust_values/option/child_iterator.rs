use crate::{
    traits::child_iterator::ChildIterator,
    values::borrowed_value_container::{
        BorrowedValueContainer, BorrowedValueContainerMut,
    },
};

impl<T: ChildIterator> ChildIterator for Option<T> {
    fn iter_children<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
        match self {
            Some(value) => value.iter_children(),
            None => None,
        }
    }
    fn iter_children_mut<'a>(
        &'a mut self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>>
    {
        match self {
            Some(value) => value.iter_children_mut(),
            None => None,
        }
    }
}
