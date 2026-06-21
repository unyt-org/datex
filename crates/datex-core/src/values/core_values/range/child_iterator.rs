use crate::traits::child_iterator::ChildIterator;
use crate::values::core_values::range::Range;
use crate::values::value_container::ValueContainer;

impl<'a> ChildIterator<'a> for Range {
    fn iter_children(&'a self) -> impl Iterator<Item = &ValueContainer> + 'a {
        gen {
            yield self.start.as_ref();
            yield self.end.as_ref();
        }
    }
}