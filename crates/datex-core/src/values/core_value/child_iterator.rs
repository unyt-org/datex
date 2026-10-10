use crate::{
    traits::child_iterator::ChildIterator,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        core_value::CoreValue,
    },
};

impl ChildIterator for CoreValue {
    fn iter_children<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
        match self {
            CoreValue::Map(map) => map.iter_children(),
            CoreValue::List(list) => list.iter_children(),
            CoreValue::Range(range) => range.iter_children(),
            _ => None,
        }
    }

    fn iter_children_mut<'a>(
        &'a mut self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>>
    {
        match self {
            CoreValue::Map(map) => map.iter_children_mut(),
            CoreValue::List(list) => list.iter_children_mut(),
            CoreValue::Range(range) => range.iter_children_mut(),
            _ => None,
        }
    }
}
