use crate::{
    collections::HashMap,
    runtime::{
        RuntimeInternal,
        execution::{
            ExecutionError,
            execution_loop::{
                ExternalExecutionInterrupt, execution_loop,
                interrupts::InterruptProvider,
            },
        },
    },
    shared_values::observers::TransceiverId,
    values::value_container::ValueContainer,
};
use core::{cell::RefCell, fmt::Debug};
use crate::prelude::*;
use crate::runtime::execution::execution_input::ExecutionCallerMetadata;

pub struct ExecutionLoopState {
    pub iterator: Box<
        dyn Iterator<Item = Result<ExternalExecutionInterrupt, ExecutionError>>,
    >,
    pub dxb_body: Rc<RefCell<Vec<u8>>>,
    pub(crate) interrupt_provider: InterruptProvider,
}
impl ExecutionLoopState {
    pub fn new(
        dxb_body: Vec<u8>,
        runtime: Rc<RuntimeInternal>,
        slots: RuntimeExecutionStack,
        caller_metadata: ExecutionCallerMetadata,
    ) -> Self {
        let state = RuntimeExecutionState {
            runtime_internal: runtime.clone(),
            source_id: 0, // TODO #640: set proper source ID
            stack: slots,
            caller_metadata,
        };
        // TODO #641: optimize, don't clone the whole DXB body every time here
        let dxb_rc = Rc::new(RefCell::new(dxb_body.to_vec()));
        let interrupt_provider = InterruptProvider::new();
        ExecutionLoopState {
            dxb_body: dxb_rc.clone(),
            iterator: Box::new(execution_loop(
                state,
                dxb_rc,
                interrupt_provider.clone(),
            )),
            interrupt_provider,
        }
    }
}

impl Debug for ExecutionLoopState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ExecutionIterator")
            .field("dxb_body_length", &self.dxb_body.borrow().len())
            .finish()
    }
}

#[derive(Debug)]
pub struct RuntimeExecutionState {
    /// Local memory stack for current execution context.
    pub stack: RuntimeExecutionStack,
    pub runtime_internal: Rc<RuntimeInternal>,
    pub source_id: TransceiverId,
    pub caller_metadata: ExecutionCallerMetadata
}

#[derive(Debug, Default)]
pub struct RuntimeExecutionStack {
    pub values: Vec<Option<ValueContainer>>,
}

impl RuntimeExecutionStack {
    /// Pushes a value to the stack
    pub(crate) fn push(
        &mut self,
        value: ValueContainer,
    ) {
        self.values.push(Some(value));
    }


    /// Pushes multiple values to the stack
    pub(crate) fn push_multiple(
        &mut self,
        values: Vec<ValueContainer>,
    ) {
        self.values.extend(values.into_iter().map(Some));
    }

    /// Takes a slot by its index and returns its value.
    /// If the slot is not allocated or the index is out of bounds, it returns an error.
    pub(crate) fn take_slot(
        &mut self,
        index: u32,
    ) -> Result<ValueContainer, ExecutionError> {
        if let Some(slot) = self.values.get_mut(index as usize) {
            slot.take().ok_or_else(|| ExecutionError::StackValueNotAllocated(index))
        }
        else {
            Err(ExecutionError::StackOutOfBoundsAccess(index))
        }
    }

    /// Sets the value of a slot, returning the previous value if it existed.
    /// If the slot is not allocated, it returns an error.
    pub(crate) fn set_slot_value(
        &mut self,
        index: u32,
        value: ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        if let Some(slot) = self.values.get_mut(index as usize) {
            Ok(slot.replace(value))
        }
        else {
            Err(ExecutionError::StackOutOfBoundsAccess(index))
        }
    }

    /// Retrieves a reference to the value of a slot by its address.
    /// If the slot is not allocated, it returns an error.
    pub(crate) fn get_slot_value(
        &self,
        index: u32,
    ) -> Result<&ValueContainer, ExecutionError> {
        if let Some(slot) = self.values.get(index as usize) {
            slot.as_ref().ok_or_else(|| ExecutionError::StackValueNotAllocated(index))
        }
        else {
            Err(ExecutionError::StackOutOfBoundsAccess(index))
        }
    }

    /// Retrieves a mutable reference to the value of a slot by its address.
    /// If the slot is not allocated, it returns an error.
    pub(crate) fn get_slot_value_mut(
        &mut self,
        index: u32,
    ) -> Result<&mut ValueContainer, ExecutionError> {
        if let Some(slot) = self.values.get_mut(index as usize) {
            slot.as_mut().ok_or_else(|| ExecutionError::StackValueNotAllocated(index))
        }
        else {
            Err(ExecutionError::StackOutOfBoundsAccess(index))
        }
    }
}
