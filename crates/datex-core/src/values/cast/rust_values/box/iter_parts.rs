use crate::traits::iter_parts::IterParts;

impl<T> IterParts for Box<T>
where
    T: IterParts,
{
    fn iter_list_parts<'a>(
		&'a self,
	) -> Option<Box<dyn Iterator<Item = crate::values::borrowed_value_container::BorrowedValueContainer<'a>> + 'a>>
{
        self.as_ref().iter_list_parts()
    }
	fn iter_list_parts_mut<'a>(
		&'a mut self,
	) -> Option<Box<dyn Iterator<Item = crate::values::borrowed_value_container::BorrowedValueContainerMut<'a>> + 'a>>
{
        self.as_mut().iter_list_parts_mut()
    }
}
