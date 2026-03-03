use crate::{
    dxb_parser::body::DXBParserError,
    network::com_hub::network_response::ResponseError,
    runtime::execution::execution_loop::state::ExecutionLoopState,
    shared_values::shared_container::{
        AccessError, AssignmentError, SharedValueCreationError,
    },
    types::error::IllegalTypeError,
    values::value_container::{ValueContainer, ValueError},
};
use core::fmt::Display;

use crate::prelude::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidProgramError {
    // any unterminated sequence, e.g. missing key in key-value pair
    UnterminatedSequence,
    MissingRemoteExecutionReceiver,
    ExpectedTypeValue,
    ExpectedValue,
    ExpectedInstruction,
    ExpectedRegularInstruction,
    ExpectedTypeInstruction,
}

impl Display for InvalidProgramError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InvalidProgramError::UnterminatedSequence => {
                core::write!(f, "Unterminated sequence")
            }
            InvalidProgramError::MissingRemoteExecutionReceiver => {
                core::write!(f, "Missing remote execution receiver")
            }
            InvalidProgramError::ExpectedTypeValue => {
                core::write!(f, "Expected a type value")
            }
            InvalidProgramError::ExpectedValue => {
                core::write!(f, "Expected a value")
            }
            InvalidProgramError::ExpectedRegularInstruction => {
                core::write!(f, "Expected a regular instruction")
            }
            InvalidProgramError::ExpectedTypeInstruction => {
                core::write!(f, "Expected a type instruction")
            }
            InvalidProgramError::ExpectedInstruction => {
                core::write!(f, "Expected an instruction")
            }
        }
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    DXBParserError(DXBParserError),
    ValueError(ValueError),
    InvalidProgram(InvalidProgramError),
    AccessError(AccessError),
    Unknown,
    NotImplemented(String),
    SlotNotAllocated(u32),
    SlotNotInitialized(u32),
    RequiresAsyncExecution,
    ResponseError(ResponseError),
    IllegalTypeError(IllegalTypeError),
    ReferenceNotFound,
    InvalidUnbox,
    InvalidTypeCast,
    ExpectedTypeValue,
    ReferenceToNonSharedValue,
    MutableReferenceToNonMutableValue,
    AssignmentError(AssignmentError),
    ReferenceCreationError(SharedValueCreationError),
    IntermediateResultWithState(
        Option<ValueContainer>,
        Option<ExecutionLoopState>,
    ),
    InvalidApply,
}
impl From<SharedValueCreationError> for ExecutionError {
    fn from(error: SharedValueCreationError) -> Self {
        ExecutionError::ReferenceCreationError(error)
    }
}

impl From<AccessError> for ExecutionError {
    fn from(error: AccessError) -> Self {
        ExecutionError::AccessError(error)
    }
}

impl From<DXBParserError> for ExecutionError {
    fn from(error: DXBParserError) -> Self {
        ExecutionError::DXBParserError(error)
    }
}

impl From<ValueError> for ExecutionError {
    fn from(error: ValueError) -> Self {
        ExecutionError::ValueError(error)
    }
}

impl From<IllegalTypeError> for ExecutionError {
    fn from(error: IllegalTypeError) -> Self {
        ExecutionError::IllegalTypeError(error)
    }
}

impl From<InvalidProgramError> for ExecutionError {
    fn from(error: InvalidProgramError) -> Self {
        ExecutionError::InvalidProgram(error)
    }
}

impl From<ResponseError> for ExecutionError {
    fn from(error: ResponseError) -> Self {
        ExecutionError::ResponseError(error)
    }
}

impl From<AssignmentError> for ExecutionError {
    fn from(error: AssignmentError) -> Self {
        ExecutionError::AssignmentError(error)
    }
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ExecutionError::ReferenceCreationError(err) => {
                core::write!(f, "Reference from value container error: {err}")
            }
            ExecutionError::ReferenceNotFound => {
                core::write!(f, "Reference not found")
            }
            ExecutionError::DXBParserError(err) => {
                core::write!(f, "Parser error: {err}")
            }
            ExecutionError::Unknown => {
                core::write!(f, "Unknown execution error")
            }
            ExecutionError::ValueError(err) => {
                core::write!(f, "Value error: {err}")
            }
            ExecutionError::InvalidProgram(err) => {
                core::write!(f, "Invalid program error: {err}")
            }
            ExecutionError::NotImplemented(msg) => {
                core::write!(f, "Not implemented: {msg}")
            }
            ExecutionError::SlotNotAllocated(address) => {
                core::write!(
                    f,
                    "Tried to access unallocated slot at address {address}"
                )
            }
            ExecutionError::SlotNotInitialized(address) => {
                core::write!(
                    f,
                    "Tried to access uninitialized slot at address {address}"
                )
            }
            ExecutionError::RequiresAsyncExecution => {
                core::write!(f, "Program must be executed asynchronously")
            }
            ExecutionError::ResponseError(err) => {
                core::write!(f, "Response error: {err}")
            }
            ExecutionError::IllegalTypeError(err) => {
                core::write!(f, "Illegal type: {err}")
            }
            ExecutionError::InvalidUnbox => {
                core::write!(f, "Tried to unbox a non-reference value")
            }
            ExecutionError::AssignmentError(err) => {
                core::write!(f, "Assignment error: {err}")
            }
            ExecutionError::InvalidTypeCast => {
                core::write!(f, "Invalid type cast")
            }
            ExecutionError::ExpectedTypeValue => {
                core::write!(f, "Expected a type value")
            }
            ExecutionError::AccessError(err) => {
                core::write!(f, "Access error: {err}")
            }
            ExecutionError::IntermediateResultWithState(
                value_opt,
                state_opt,
            ) => {
                core::write!(
                    f,
                    "Execution produced an intermediate result: {:?} with state: {:?}",
                    value_opt,
                    state_opt
                )
            }
            ExecutionError::InvalidApply => {
                core::write!(f, "Invalid apply operation")
            }
            ExecutionError::ReferenceToNonSharedValue => {
                core::write!(
                    f,
                    "Tried to create a reference to a non-shared value"
                )
            }
            ExecutionError::MutableReferenceToNonMutableValue => {
                core::write!(
                    f,
                    "Tried to create a mutable reference to a non-mutable value"
                )
            }
        }
    }
}
