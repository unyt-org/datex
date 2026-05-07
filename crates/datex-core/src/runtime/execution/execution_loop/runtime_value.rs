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
    /// Call the provided closure with a reference to the underlying `ValueContainer`.
    /// If the `RuntimeValue` is a slot address, it retrieves the value from the runtime state.
    pub fn with_mut_value_container<F, R>(
        &mut self,
        slots: &mut RuntimeExecutionStack,
        f: F,
    ) -> Result<R, ExecutionError>
    where
        F: FnOnce(&mut ValueContainer) -> R,
    {
        match self {
            RuntimeValue::ValueContainer(vc) => Ok(f(vc)),
            RuntimeValue::StackValue(addr) => {
                let slot_value = slots.get_stack_value_mut(*addr)?;
                Ok(f(slot_value))
            }
        }
    }

    /// Creates an owned `ValueContainer` from the `RuntimeValue`.
    /// This possibly involves cloning the value if it is stored in a slot.
    /// Do not use this method if you want to work on the actual value without cloning it.
    #[deprecated(note = "value container clone should not be used")]
    pub fn into_cloned_value_container(
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
