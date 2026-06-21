use crate::traits::child_iterator::ChildIterator;
use crate::values::core_value::CoreValue;
use crate::values::value_container::ValueContainer;

impl<'a> ChildIterator<'a> for CoreValue {
    fn iter_children(&self) -> Box<dyn Iterator<Item = &ValueContainer>> {
        match self {
            CoreValue::Map(map) => map.iter_children(),
            CoreValue::List(list) => list.iter_children(),
            CoreValue::Range(range) => range.iter_children(),
            _ => Box::new(core::iter::empty()),
        }
    }
}