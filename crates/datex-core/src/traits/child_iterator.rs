use crate::values::value_container::ValueContainer;

/// Trait for types that can provide an iterator over their child value containers
pub trait ChildIterator<'a> {
    fn iter_children(&'a self)
    -> impl Iterator<Item = &'a ValueContainer> + 'a;
    fn iter_children_mut(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut ValueContainer> + 'a;
}
