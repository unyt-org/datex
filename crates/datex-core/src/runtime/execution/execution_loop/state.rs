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
        slots: RuntimeExecutionSlots,
        caller_metadata: ExecutionCallerMetadata,
    ) -> Self {
        let state = RuntimeExecutionState {
            runtime_internal: runtime.clone(),
            source_id: 0, // TODO #640: set proper source ID
            slots,
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
    /// Local memory slots for current execution context.
    /// TODO #643: replace this with a local stack and deprecate local slots?
    pub slots: RuntimeExecutionSlots,
    pub runtime_internal: Rc<RuntimeInternal>,
    pub source_id: TransceiverId,
    pub caller_metadata: ExecutionCallerMetadata
}

#[derive(Debug, Default)]
pub struct RuntimeExecutionSlots {
    pub slots: HashMap<u32, Option<ValueContainer>>,
}

impl RuntimeExecutionSlots {
    /// Allocates a new slot with the given slot address.
    pub(crate) fn allocate_slot(
        &mut self,
        address: u32,
        value: Option<ValueContainer>,
    ) {
        self.slots.insert(address, value);
    }

    /// Drops a slot by its address, returning the value if it existed.
    /// If the slot is not allocated, it returns an error.
    pub(crate) fn drop_slot(
        &mut self,
        address: u32,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.slots
            .remove(&address)
            .ok_or(())
            .map_err(|_| ExecutionError::SlotNotAllocated(address))
    }

    /// Sets the value of a slot, returning the previous value if it existed.
    /// If the slot is not allocated, it returns an error.
    pub(crate) fn set_slot_value(
        &mut self,
        address: u32,
        value: ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.slots
            .insert(address, Some(value))
            .ok_or(())
            .map_err(|_| ExecutionError::SlotNotAllocated(address))
    }

    /// Retrieves a reference to the value of a slot by its address.
    /// If the slot is not allocated, it returns an error.
    pub(crate) fn get_slot_value(
        &self,
        address: u32,
    ) -> Result<&ValueContainer, ExecutionError> {
        self.slots
            .get(&address)
            .and_then(|inner| inner.as_ref())
            .ok_or_else(|| ExecutionError::SlotNotAllocated(address))
    }

    /// Retrieves a mutable reference to the value of a slot by its address.
    /// If the slot is not allocated, it returns an error.
    pub(crate) fn get_slot_value_mut(
        &mut self,
        address: u32,
    ) -> Result<&mut ValueContainer, ExecutionError> {
        self.slots
            .get_mut(&address)
            .and_then(|inner| inner.as_mut())
            .ok_or_else(|| ExecutionError::SlotNotAllocated(address))
    }
}
