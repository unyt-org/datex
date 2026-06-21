use crate::traits::child_iterator::ChildIterator;
use crate::values::core_values::map::{BorrowedMapKey, Map};
use crate::values::value_container::ValueContainer;

impl<'a> ChildIterator<'a> for Map {
    fn iter_children(&'a self) -> Box<dyn Iterator<Item = &ValueContainer> + 'a> {
        Box::new(gen {
            for (key, value) in self.iter() {
                match key {
                    BorrowedMapKey::Value(v) => yield v,
                    _ => {}
                };
                yield value;
            }
        })
    }
}