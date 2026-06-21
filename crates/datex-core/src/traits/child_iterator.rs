use crate::values::value_container::ValueContainer;

/// Trait for types that can provide an iterator over their child value containers
pub trait ChildIterator<'a> {
    fn iter_children(&'a self) -> Box<dyn Iterator<Item = &ValueContainer> + 'a>;
}