use crate::{
    traits::child_iterator::ChildIterator,
    values::{
        core_values::map::{BorrowedMapKey, BorrowedMutMapKey, Map},
        value_container::ValueContainer,
    },
};

impl<'a> ChildIterator<'a> for Map {
    fn iter_children(
        &'a self,
    ) -> impl Iterator<Item = &'a ValueContainer> + 'a {
        gen {
            for (key, value) in self.iter() {
                if let BorrowedMapKey::Value(v) = key {
                    yield v
                };
                yield value;
            }
        }
    }

    fn iter_children_mut(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut ValueContainer> + 'a {
        gen {
            for (key, value) in self.into_iter() {
                if let BorrowedMutMapKey::Value(v) = key {
                    yield v
                };
                yield value;
            }
        }
    }
}
