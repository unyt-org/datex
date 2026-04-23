use core::cell::RefCell;

use crate::values::value_container::ValueContainer;

use crate::{
    global::protocol_structures::instruction_data::{
        RawBuiltinPointerAddress, RawLocalPointerAddress,
        RawRemotePointerAddress,
    },
    prelude::*,
    shared_values::shared_containers::{
        ReferenceMutability, SharedContainerMutability,
    },
};

#[derive(Debug)]
pub enum ExecutionInterrupt {
    // used for intermediate results in unbounded scopes
    SetActiveValue(Option<ValueContainer>),
    /// yields an external interrupt to be handled by the execution loop caller (for I/O operations, pointer resolution, remote execution, etc.)
    External(ExternalExecutionInterrupt),
}

#[derive(Debug)]
pub enum ExternalExecutionInterrupt {
    Result(Option<ValueContainer>),
    GetReferenceToRemotePointer(RawRemotePointerAddress, ReferenceMutability),
    GetReferenceToLocalPointer(RawLocalPointerAddress),
    GetReferenceToBuiltinPointer(RawBuiltinPointerAddress),
    RemoteExecution(ValueContainer, Vec<u8>),
    Apply(ValueContainer, Vec<ValueContainer>),
    /// Request to move a list of pointers from the current caller endpoint to the local endpoint
    RequestMove(Vec<(SharedContainerMutability, RawLocalPointerAddress)>),
    /// Move a list of pointers from the local endpoint to the caller
    Move(Vec<(RawLocalPointerAddress, RawLocalPointerAddress)>),
}

#[derive(Debug)]
pub enum InterruptResult {
    ResolvedValue(Option<ValueContainer>),
    ResolvedValues(Vec<ValueContainer>),
}

#[derive(Debug, Clone)]
pub struct InterruptProvider {
    result: Rc<RefCell<Option<InterruptResult>>>,
}

impl Default for InterruptProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl InterruptProvider {
    pub fn new() -> Self {
        Self {
            result: Rc::new(RefCell::new(None)),
        }
    }

    pub fn provide_result(&self, result: InterruptResult) {
        *self.result.borrow_mut() = Some(result);
    }

    pub fn take_result(&self) -> Option<InterruptResult> {
        self.result.borrow_mut().take()
    }
}
