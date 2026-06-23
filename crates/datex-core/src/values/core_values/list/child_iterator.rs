use crate::{
    traits::child_iterator::ChildIterator,
    values::{core_values::list::List, value_container::ValueContainer},
};

impl<'a> ChildIterator<'a> for List {
    fn iter_children(
        &'a self,
    ) -> impl Iterator<Item = &'a ValueContainer> + 'a {
        gen {
            for value in self.iter() {
                yield value;
            }
        }
    }
}
