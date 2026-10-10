use crate::traits::child_iterator::ChildIterator;

impl<T> ChildIterator for Box<T>
where
    T: ChildIterator,
{
    fn iter_children<'a>(
		&'a self,
	) -> Option<Box<dyn Iterator<Item = crate::values::borrowed_value_container::BorrowedValueContainer<'a>> + 'a>>
{
        self.as_ref().iter_children()
    }
	fn iter_children_mut<'a>(
		&'a mut self,
	) -> Option<Box<dyn Iterator<Item = crate::values::borrowed_value_container::BorrowedValueContainerMut<'a>> + 'a>>
{
        self.as_mut().iter_children_mut()
    }
}
