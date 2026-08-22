use crate::{
    traits::child_iterator::ChildIterator,
    values::{core_values::range::Range, value_container::ValueContainer},
};

impl<'a> ChildIterator<'a> for Range {
    fn iter_children(&self) -> impl Iterator<Item = &ValueContainer> {
        gen {
            yield self.start.as_ref();
            yield self.end.as_ref();
        }
    }
}
