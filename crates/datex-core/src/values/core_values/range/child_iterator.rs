use crate::{
    traits::child_iterator::ChildIterator,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        core_values::range::Range,
    },
};

impl ChildIterator for Range {
    fn iter_children<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
        Some(Box::new(gen {
            yield self.start.as_ref().into();
            yield self.end.as_ref().into();
        }))
    }

    fn iter_children_mut<'a>(
        &'a mut self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>>
    {
        Some(Box::new(gen {
            yield self.start.as_mut().into();
            yield self.end.as_mut().into();
        }))
    }
}
