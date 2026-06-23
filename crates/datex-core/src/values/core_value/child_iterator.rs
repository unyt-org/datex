use crate::{
    traits::child_iterator::ChildIterator,
    values::{core_value::CoreValue, value_container::ValueContainer},
};

impl<'a> ChildIterator<'a> for CoreValue {
    fn iter_children(&self) -> impl Iterator<Item = &ValueContainer> {
        gen {
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
}
