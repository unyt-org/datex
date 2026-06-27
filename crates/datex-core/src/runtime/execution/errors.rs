use crate::{
    dxb_parser::body::DXBParserError,
    global::protocol_structures::instruction_data::StackIndex,
    libs::core::core_lib_id::CoreLibIdIndex,
    network::com_hub::network_response::ResponseError,
    prelude::*,
    runtime::execution::execution_loop::state::ExecutionLoopState,
    shared_values::errors::{
        AccessError, AssignmentError, SharedValueCreationError,
    },
    traits::apply::ApplyError,
    types::error::IllegalTypeError,
    value_updates::errors::UpdateError,
    values::value_container::{ValueContainer, error::ValueError},
};
use core::fmt::Display;
use crate::runtime::cache::shared_values_cache::CacheValueRetrievalError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidProgramError {
    // any unterminated sequence, e.g. missing key in key-value pair
    UnterminatedSequence,
    MissingRemoteExecutionReceiver,
    ExpectedTypeValue,
    ExpectedAliasType,
    ExpectedValue,
    ExpectedList,
    ExpectedInstruction,
    ExpectedRegularInstruction,
    ExpectedTypeInstruction,
    InvalidCoreLibId(CoreLibIdIndex),
}

impl Display for InvalidProgramError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InvalidProgramError::ExpectedAliasType => {
                core::write!(f, "Expected an alias type definition")
            }
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
            InvalidProgramError::ExpectedList => {
                core::write!(f, "Expected a list")
            }
            InvalidProgramError::InvalidCoreLibId(core_lib_id) => {
                core::write!(f, "Invalid core library id: {core_lib_id}")
            }
        }
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    Unspecified(String), // TODO: replace with nested stack trace exceptions
    DXBParserError(DXBParserError),
    ValueError(ValueError),
    InvalidProgram(InvalidProgramError),
    AccessError(AccessError),
    CacheValueRetrievalError(CacheValueRetrievalError),
    UpdateError(UpdateError),
    Unknown,
    NotImplemented(String),
    StackValueNotAllocated(StackIndex),
    StackOutOfBoundsAccess(StackIndex),
    InternalSlotDoesNotExist(u32),
    RequiresAsyncExecution,
    ResponseError(ResponseError),
    IllegalTypeError(IllegalTypeError),
    ReferenceNotFound,
    InvalidUnbox,
    InvalidTypeCast,
    ExpectedTypeValue,
    InvalidSharedValueType,
    ExpectedSharedValue,
    ExpectedLocalValue,
    ExpectedOwnedSharedValue,
    MutableReferenceToNonMutableValue,
    AssignmentError(AssignmentError),
    ReferenceCreationError(SharedValueCreationError),
    IntermediateResultWithState(
        Option<ValueContainer>,
        Option<ExecutionLoopState>,
    ),
    ApplyError(ApplyError),
    UnauthorizedMove,
    InvalidMove,
    MoveToMultipleEndpoints,
}

impl From<ApplyError> for ExecutionError {
    fn from(error: ApplyError) -> Self {
        ExecutionError::ApplyError(error)
    }
}

impl From<SharedValueCreationError> for ExecutionError {
    fn from(error: SharedValueCreationError) -> Self {
        ExecutionError::ReferenceCreationError(error)
    }
}

impl From<CacheValueRetrievalError> for ExecutionError {
    fn from(err: CacheValueRetrievalError) -> Self {
        ExecutionError::CacheValueRetrievalError(err)
    }
}

impl From<AccessError> for ExecutionError {
    fn from(error: AccessError) -> Self {
        ExecutionError::AccessError(error)
    }
}

impl From<UpdateError> for ExecutionError {
    fn from(error: UpdateError) -> Self {
        ExecutionError::UpdateError(error)
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
            ExecutionError::ApplyError(err) => {
                core::write!(f, "Apply error: {err}")
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
            ExecutionError::InvalidSharedValueType => {
                core::write!(f, "Invalid shared value type")
            }
            ExecutionError::AccessError(err) => {
                core::write!(f, "Access error: {err}")
            }
            ExecutionError::CacheValueRetrievalError(err) => {
                core::write!(f, "Cache value retrieval error: {err}")
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
            ExecutionError::UnauthorizedMove => {
                core::write!(f, "Unauthorized move of shared pointer")
            }
            ExecutionError::InvalidMove => {
                core::write!(f, "Invalid move of shared pointer")
            }
            ExecutionError::MoveToMultipleEndpoints => {
                core::write!(f, "Illegal move to multiple endpoints")
            }
            ExecutionError::ExpectedSharedValue => {
                core::write!(
                    f,
                    "Expected a shared value, but got a local value"
                )
            }
            ExecutionError::ExpectedLocalValue => {
                core::write!(
                    f,
                    "Expected a local value, but got a shared value"
                )
            }
            ExecutionError::ExpectedOwnedSharedValue => {
                core::write!(
                    f,
                    "Expected an owned shared value, but got a non-owned shared value"
                )
            }
            ExecutionError::MutableReferenceToNonMutableValue => {
                core::write!(
                    f,
                    "Tried to create a mutable reference to a non-mutable value"
                )
            }
            ExecutionError::StackValueNotAllocated(index) => {
                core::write!(
                    f,
                    "Tried to access unallocated stack value at index {index}"
                )
            }
            ExecutionError::StackOutOfBoundsAccess(index) => {
                core::write!(
                    f,
                    "Tried to access out of bounds stack value at index {index}"
                )
            }
            ExecutionError::InternalSlotDoesNotExist(index) => {
                core::write!(f, "Internal slot does not exist at index {index}")
            }
            ExecutionError::UpdateError(err) => {
                core::write!(f, "Value update error: {err}")
            }
            ExecutionError::Unspecified(msg) => {
                core::write!(f, "Unspecified error: {msg}")
            }
        }
    }
}
