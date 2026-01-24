use core::fmt::Display;

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
pub enum InterfaceCreateError {
    InterfaceError(ComInterfaceError),
    InterfaceAddError(InterfaceAddError),
    InterfaceCreationRequiresAsyncContext,
    InterfaceTypeNotRegistered(String),
    InterfaceOpenFailed,
    SetupDataParseError,
    Timeout,
    InvalidSetupData(String),
}

impl InterfaceCreateError {
    pub fn invalid_setup_data<T: Display>(details: T) -> Self {
        InterfaceCreateError::InvalidSetupData(details.to_string())
    }
}

impl From<InterfaceAddError> for InterfaceCreateError {
    fn from(err: InterfaceAddError) -> Self {
        InterfaceCreateError::InterfaceAddError(err)
    }
}

impl From<ComInterfaceError> for InterfaceCreateError {
    fn from(err: ComInterfaceError) -> Self {
        InterfaceCreateError::InterfaceError(err)
    }
}

impl Display for InterfaceCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InterfaceCreateError::Timeout => {
                core::write!(f, "InterfaceCreationError: Timeout")
            }
            InterfaceCreateError::InterfaceError(_msg) => {
                core::write!(f, "InterfaceCreationError: ComInterfaceError")
            }
            InterfaceCreateError::InterfaceCreationRequiresAsyncContext => {
                core::write!(
                    f,
                    "InterfaceCreationError: Interface creation requires async context"
                )
            }
            InterfaceCreateError::InterfaceTypeNotRegistered(ty) => {
                core::write!(
                    f,
                    "InterfaceCreationError: Interface type '{}' is not registered",
                    ty
                )
            }
            InterfaceCreateError::InterfaceOpenFailed => {
                core::write!(
                    f,
                    "InterfaceCreationError: Failed to open interface"
                )
            }
            InterfaceCreateError::SetupDataParseError => {
                core::write!(
                    f,
                    "InterfaceCreationError: Setup data parse error"
                )
            }
            InterfaceCreateError::InvalidSetupData(details) => {
                core::write!(
                    f,
                    "InterfaceCreationError: Invalid setup data - {}",
                    details
                )
            }
            InterfaceCreateError::InterfaceAddError(add_err) => {
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
