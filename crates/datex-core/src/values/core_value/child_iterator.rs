use crate::{
    traits::child_iterator::ChildIterator,
    values::{core_value::CoreValue, value_container::ValueContainer},
};

impl<'a> ChildIterator<'a> for CoreValue {
    fn iter_children(&self) -> impl Iterator<Item = &ValueContainer> {
        gen move {
            match self {
                CoreValue::Map(map) => {
                    for value in map.iter_children() {
                        yield value;
                    }
                }
                CoreValue::List(list) => {
                    for value in list.iter_children() {
                        yield value;
                    }
                }
                CoreValue::Range(range) => {
                    for value in range.iter_children() {
                        yield value;
                    }
                }
                _ => {}
            }
        }
    }

    fn iter_children_mut(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut ValueContainer> + 'a {
        gen move {
            match self {
                CoreValue::Map(map) => {
                    for value in map.iter_children_mut() {
                        yield value;
                    }
                }
                CoreValue::List(list) => {
                    for value in list.iter_children_mut() {
                        yield value;
                    }
                }
                CoreValue::Range(range) => {
                    for value in range.iter_children_mut() {
                        yield value;
                    }
                }
                _ => {}
            }
        }
    }
}
