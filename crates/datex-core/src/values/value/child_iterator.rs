use crate::{
    traits::child_iterator::ChildIterator,
    values::{value::Value, value_container::ValueContainer},
};

impl<'a> ChildIterator<'a> for Value {
    fn iter_children(&self) -> impl Iterator<Item = &ValueContainer> {
        match self {
            Value::Core(core_value) => core_value.inner.iter_children(),
            Value::Native(native_value) => todo!()
        }
    }

    fn iter_children_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut ValueContainer> {
        match self {
            Value::Core(core_value) => core_value.inner.iter_children_mut(),
            Value::Native(native_value) => todo!()
        }
    }
}
