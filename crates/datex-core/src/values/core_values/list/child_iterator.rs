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

    fn iter_children_mut(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut ValueContainer> + 'a {
        gen {
            for value in self.iter_mut() {
                yield value;
            }
        }
    }
}
