use crate::{
    traits::child_iterator::ChildIterator,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        core_values::map::{BorrowedMapKey, BorrowedMutMapKey, Map},
    },
};

impl ChildIterator for Map {
    fn iter_children<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
        Some(Box::new(gen move {
            for (key, value) in self.iter() {
                if let BorrowedMapKey::Value(v) = key {
                    yield v.into();
                };
                yield value.into();
            }
        }))
    }

    fn iter_children_mut<'a>(
        &'a mut self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>>
    {
        Some(Box::new(gen move {
            for (key, value) in self.into_iter() {
                if let BorrowedMutMapKey::Value(v) = key {
                    yield v.into();
                };
                yield value.into();
            }
        }))
    }
}
