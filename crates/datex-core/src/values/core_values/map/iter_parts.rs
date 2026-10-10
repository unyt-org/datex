use crate::{
    traits::iter_parts::IterParts,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        core_values::map::{BorrowedMapKey, BorrowedMutMapKey, Map},
    },
};

impl IterParts for Map {
    fn iter_map_parts<'a>(
        &'a self,
    ) -> Option<
        Box<
            dyn Iterator<
                    Item = (
                        BorrowedValueContainer<'a>,
                        BorrowedValueContainer<'a>,
                    ),
                > + 'a,
        >,
    > {
        Some(Box::new(gen move {
            for (key, value) in self.iter() {
                if let BorrowedMapKey::Value(v) = key {
                    v.into();
                };
                yield value.into();
            }
        }))
    }

    fn iter_map_parts_mut<'a>(
        &'a mut self,
    ) -> Option<
        Box<
            dyn Iterator<
                    Item = (
                        BorrowedValueContainer<'a>,
                        BorrowedValueContainerMut<'a>,
                    ),
                > + 'a,
        >,
    > {
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
