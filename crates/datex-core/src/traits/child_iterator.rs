use crate::values::borrowed_value_container::{
    BorrowedValueContainer, BorrowedValueContainerMut,
};

/// Trait for types that can provide an iterator over their child value containers
pub trait ChildIterator {
    fn iter_children<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
        None
    }
    fn iter_children_mut<'a>(
        &'a mut self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>>
    {
        None
    }
}
