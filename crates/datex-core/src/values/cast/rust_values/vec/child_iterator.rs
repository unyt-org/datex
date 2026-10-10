use crate::{
    traits::{
        child_iterator::ChildIterator,
        convert_value_container::ConvertValueContainer,
    },
    values::borrowed_value_container::{
        BorrowedValueContainer, BorrowedValueContainerMut,
    },
};

impl<T: ConvertValueContainer> ChildIterator for Vec<T> {
    fn iter_children<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
        Some(Box::new(
            self.iter().map(|item| item.as_borrowed_value_container()),
        ))
    }
    fn iter_children_mut<'a>(
        &'a mut self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>>
    {
        Some(Box::new(
            self.iter_mut()
                .map(|item| item.as_borrowed_value_container_mut()),
        ))
    }
}
