use crate::{
    global::protocol_structures::instruction_data::StackIndex,
    runtime::execution::{
        ExecutionError,
        execution_loop::{
            internal_slots::get_stack_value,
            state::{RuntimeExecutionStack, RuntimeExecutionState},
        },
    },
    values::value_container::ValueContainer,
};

/// Represents a value in the runtime execution context, which can either be a direct
/// `ValueContainer` or a reference to a slot address where the value is stored.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    ValueContainer(ValueContainer),
    StackValue(StackIndex),
}

impl From<ValueContainer> for RuntimeValue {
    fn from(value: ValueContainer) -> Self {
        RuntimeValue::ValueContainer(value)
    }
}

impl From<StackIndex> for RuntimeValue {
    fn from(index: StackIndex) -> Self {
        RuntimeValue::StackValue(index)
    }
}

impl RuntimeValue {
    pub fn as_value_container<'a>(
        &'a self,
        slots: &'a RuntimeExecutionStack,
    ) -> Result<&'a ValueContainer, ExecutionError> {
        match self {
            RuntimeValue::ValueContainer(vc) => Ok(vc),
            RuntimeValue::StackValue(index) => slots.get_stack_value(*index),
        }
    }

    pub fn as_value_container_mut<'a>(
        &'a mut self,
        slots: &'a mut RuntimeExecutionStack,
    ) -> Result<&'a mut ValueContainer, ExecutionError> {
        match self {
            RuntimeValue::ValueContainer(vc) => Ok(vc),
            RuntimeValue::StackValue(index) => {
                slots.get_stack_value_mut(*index)
            }
        }
    }

    /// Creates an owned `ValueContainer` from the `RuntimeValue`.
    /// This possibly involves cloning the value if it is stored in a slot.
    /// Do not use this method if you want to work on the actual value without cloning it.
    #[deprecated(note = "value container clone should not be used")]
    pub fn into_potentially_cloned_value_container(
        self,
        state: &RuntimeExecutionState,
    ) -> Result<ValueContainer, ExecutionError> {
        match self {
            RuntimeValue::ValueContainer(vc) => Ok(vc),
            RuntimeValue::StackValue(addr) => {
                Ok(get_stack_value(state, addr)?.clone())
            }
        }
    }

    /// Creates an owned `ValueContainer` from the `RuntimeValue`.
    /// If the runtime value is inside a slot, it is popped
    pub fn into_value_container(
        self,
        state: &mut RuntimeExecutionState,
    ) -> Result<ValueContainer, ExecutionError> {
        match self {
            RuntimeValue::ValueContainer(vc) => Ok(vc),
            RuntimeValue::StackValue(addr) => {
                Ok(state.stack.take_stack_value(addr)?)
            }
        }
    }
}
