use crate::traits::child_iterator::ChildIterator;
use crate::values::core_values::range::Range;
use crate::values::value_container::ValueContainer;

impl<'a> ChildIterator<'a> for Range {
    fn iter_children(&'a self) -> Box<dyn Iterator<Item = &ValueContainer> + 'a> {
        Box::new(gen {
            yield &self.start;
            yield &self.end;
        })
    }
}