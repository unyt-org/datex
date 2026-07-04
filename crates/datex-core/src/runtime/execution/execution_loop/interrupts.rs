use core::cell::RefCell;

use crate::values::value_container::ValueContainer;

use crate::{
    core_compiler::core_compilation_context::DXBWithSharedValues,
    libs::core::core_lib_id::CoreLibId,
    prelude::*,
    shared_values::{
        ReferenceMutability, RemotePointerAddress, SelfOwnedPointerAddress,
        SharedContainerMutability,
    },
    values::core_values::endpoint::Endpoint,
};
use crate::shared_values::SharedContainer;

#[derive(Debug)]
pub enum ExecutionInterrupt {
    // used for intermediate results in unbounded scopes
    SetActiveValue(Option<ValueContainer>),
    TakeActiveValue,
    /// yields an external interrupt to be handled by the execution loop caller (for I/O operations, pointer resolution, remote execution, etc.)
    External(ExternalExecutionInterrupt),
}

#[derive(Debug)]
pub enum ExternalExecutionInterrupt {
    Result(Option<ValueContainer>),
    GetReferenceToRemotePointer(RemotePointerAddress, ReferenceMutability),
    GetReferenceToLocalPointer(SelfOwnedPointerAddress),
    GetCoreLibValue(CoreLibId),
    RemoteExecution {
        input: DXBWithSharedValues,
        receivers: Vec<Endpoint>,
    },
    Apply(ValueContainer, Vec<ValueContainer>),
    /// Request to move a list of pointers from the current caller endpoint to the local endpoint
    RequestMove(Vec<(SharedContainerMutability, SelfOwnedPointerAddress)>),
    /// Move a list of pointers from the local endpoint to the caller
    ConfirmMoves(Vec<(SelfOwnedPointerAddress, SelfOwnedPointerAddress)>),
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
