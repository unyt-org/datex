use core::fmt::Display;
use crate::stdlib::string::String;
use crate::stdlib::string::ToString;
use crate::network::com_interfaces::com_interface::error::ComInterfaceError;

#[derive(Debug, PartialEq)]
pub enum InterfaceAddError {
    InterfaceAlreadyExists,
    InvalidInterfaceDirectionForFallbackInterface,
}

impl Display for InterfaceAddError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InterfaceAddError::InterfaceAlreadyExists => {
                core::write!(f, "InterfaceAddError: Interface already exists")
            }
            InterfaceAddError::InvalidInterfaceDirectionForFallbackInterface => {
                core::write!(
                    f,
                    "InterfaceAddError: Invalid interface direction for fallback interface"
                )
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ComInterfaceCreateError {
    InterfaceError(ComInterfaceError),
    InterfaceAddError(InterfaceAddError),
    InterfaceCreationRequiresAsyncContext,
    InterfaceTypeNotRegistered(String),
    InterfaceOpenFailed,
    SetupDataParseError,
    Timeout,
    InvalidSetupData(String),
}

impl ComInterfaceCreateError {
    pub fn invalid_setup_data<T: Display>(details: T) -> Self {
        ComInterfaceCreateError::InvalidSetupData(details.to_string())
    }
}

impl From<InterfaceAddError> for ComInterfaceCreateError {
    fn from(err: InterfaceAddError) -> Self {
        ComInterfaceCreateError::InterfaceAddError(err)
    }
}

impl From<ComInterfaceError> for ComInterfaceCreateError {
    fn from(err: ComInterfaceError) -> Self {
        ComInterfaceCreateError::InterfaceError(err)
    }
}

impl Display for ComInterfaceCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ComInterfaceCreateError::Timeout => {
                core::write!(f, "InterfaceCreationError: Timeout")
            }
            ComInterfaceCreateError::InterfaceError(_msg) => {
                core::write!(f, "InterfaceCreationError: ComInterfaceError")
            }
            ComInterfaceCreateError::InterfaceCreationRequiresAsyncContext => {
                core::write!(
                    f,
                    "InterfaceCreationError: Interface creation requires async context"
                )
            }
            ComInterfaceCreateError::InterfaceTypeNotRegistered(ty) => {
                core::write!(
                    f,
                    "InterfaceCreationError: Interface type '{}' is not registered",
                    ty
                )
            }
            ComInterfaceCreateError::InterfaceOpenFailed => {
                core::write!(
                    f,
                    "InterfaceCreationError: Failed to open interface"
                )
            }
            ComInterfaceCreateError::SetupDataParseError => {
                core::write!(
                    f,
                    "InterfaceCreationError: Setup data parse error"
                )
            }
            ComInterfaceCreateError::InvalidSetupData(details) => {
                core::write!(
                    f,
                    "InterfaceCreationError: Invalid setup data - {}",
                    details
                )
            }
            ComInterfaceCreateError::InterfaceAddError(add_err) => {
                core::write!(
                    f,
                    "InterfaceCreationError: InterfaceAddError - {}",
                    add_err
                )
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ComHubError {
    InterfaceError(ComInterfaceError),
    InterfaceCloseFailed,
    InterfaceDoesNotExist,
    InterfaceNotConnected,
    NoResponse,
    SignatureError,
}
impl From<ComInterfaceError> for ComHubError {
    fn from(err: ComInterfaceError) -> Self {
        ComHubError::InterfaceError(err)
    }
}

impl Display for ComHubError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ComHubError::InterfaceError(_msg) => {
                core::write!(f, "ComHubError: ComInterfaceError")
            }
            ComHubError::InterfaceCloseFailed => {
                core::write!(f, "ComHubError: Failed to close interface")
            }
            ComHubError::InterfaceNotConnected => {
                core::write!(f, "ComHubError: Interface not connected")
            }
            ComHubError::InterfaceDoesNotExist => {
                core::write!(f, "ComHubError: Interface does not exist")
            }
            ComHubError::NoResponse => {
                core::write!(f, "ComHubError: No response")
            }
            ComHubError::SignatureError => {
                core::write!(f, "ComHubError: CryptoError")
            }
        }
    }
}

#[derive(Debug)]
pub enum SocketEndpointRegistrationError {
    SocketEndpointAlreadyRegistered,
}
