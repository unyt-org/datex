use crate::{
    traits::child_iterator::ChildIterator,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        core_value::CoreValue,
    },
};

impl ChildIterator for CoreValue {
    fn iter_children<'a>(
        &'a self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainer<'a>> + 'a>> {
        if matches!(
            self,
            CoreValue::Map(_) | CoreValue::List(_) | CoreValue::Range(_)
        ) {
            Some(gen {
                match self {
                    CoreValue::Map(map) => {
                        for value in map
                            .iter_children()
                            .expect("Map should have children")
                        {
                            yield value;
                        }
                    }
                    CoreValue::List(list) => {
                        for value in list
                            .iter_children()
                            .expect("List should have children")
                        {
                            yield value;
                        }
                    }
                    CoreValue::Range(range) => {
                        for value in range
                            .iter_children()
                            .expect("Range should have children")
                        {
                            yield value;
                        }
                    }
                    _ => {}
                }
            })
        } else {
            None
        }
    }

    fn iter_children_mut<'a>(
        &'a mut self,
    ) -> Option<Box<dyn Iterator<Item = BorrowedValueContainerMut<'a>> + 'a>>
    {
        if matches!(
            self,
            CoreValue::Map(_) | CoreValue::List(_) | CoreValue::Range(_)
        ) {
            Some(gen move {
                match self {
                    CoreValue::Map(map) => {
                        for value in map
                            .iter_children_mut()
                            .expect("Map should have children")
                        {
                            yield value;
                        }
                    }
                    CoreValue::List(list) => {
                        for value in list
                            .iter_children_mut()
                            .expect("List should have children")
                        {
                            yield value;
                        }
                    }
                    CoreValue::Range(range) => {
                        for value in range
                            .iter_children_mut()
                            .expect("Range should have children")
                        {
                            yield value;
                        }
                    }
                    _ => {}
                }
            })
        } else {
            None
        }
    }
}
