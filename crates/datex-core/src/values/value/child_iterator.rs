use crate::{
    traits::child_iterator::ChildIterator,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        value::Value,
    },
};

impl ChildIterator for Value {
    fn iter_children<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
        match self {
            Value::Core(core_value) => core_value.inner.iter_children(),
            Value::Native(native_value) => todo!(),
        }
    }

    fn iter_children_mut<'a>(
        &'a mut self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>>
    {
        match self {
            Value::Core(core_value) => core_value.inner.iter_children_mut(),
            Value::Native(native_value) => todo!(),
        }
    }
}
