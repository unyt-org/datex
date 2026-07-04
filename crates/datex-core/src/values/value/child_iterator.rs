use crate::{
    traits::child_iterator::ChildIterator,
    values::{value::Value, value_container::ValueContainer},
};

impl<'a> ChildIterator<'a> for Value {
    fn iter_children(&self) -> impl Iterator<Item = &ValueContainer> {
        self.inner.iter_children()
    }

    fn iter_children_mut(&mut self) -> impl Iterator<Item = &mut ValueContainer> {
        self.inner.iter_children_mut()
    }
}
