use crate::traits::child_iterator::ChildIterator;
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

impl<'a> ChildIterator<'a> for Value {
    fn iter_children(&self) -> impl Iterator<Item = &ValueContainer> {
        self.inner.iter_children()
    }
}