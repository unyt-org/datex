use crate::traits::child_iterator::ChildIterator;
use crate::values::core_values::list::List;
use crate::values::value_container::ValueContainer;

impl<'a> ChildIterator<'a> for List {
    fn iter_children(&'a self) -> impl Iterator<Item = &ValueContainer> + 'a {
        gen {
            for value in self.iter() {
                yield value;
            }
        }
    }
}