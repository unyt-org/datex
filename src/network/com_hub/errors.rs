use core::fmt::Display;

use crate::network::com_interfaces::com_interface::error::ComInterfaceError;

#[derive(Debug, PartialEq)]
pub enum InterfaceCreateError {
    InterfaceError(ComInterfaceError),
    InterfaceCreationRequiresAsyncContext,
    InterfaceTypeDoesNotExist,
    InterfaceAlreadyExists,
    InvalidInterfaceDirectionForFallbackInterface,
    InterfaceOpenFailed,
    SetupDataParseError,
    InvalidSetupData(String),
}

impl InterfaceCreateError {
    pub fn invalid_setup_data<T: Display>(details: T) -> Self {
        InterfaceCreateError::InvalidSetupData(details.to_string())
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
            InterfaceCreateError::InterfaceError(_msg) => {
                core::write!(f, "InterfaceCreationError: ComInterfaceError")
            }
            InterfaceCreateError::InterfaceCreationRequiresAsyncContext => {
                core::write!(f, "InterfaceCreationError: Interface creation requires async context")
            }
            InterfaceCreateError::InterfaceTypeDoesNotExist => {
                core::write!(f, "InterfaceCreationError: Interface type does not exist")
            }
            InterfaceCreateError::InterfaceAlreadyExists => {
                core::write!(f, "InterfaceCreationError: Interface already exists")
            }
            InterfaceCreateError::InvalidInterfaceDirectionForFallbackInterface => {
                core::write!(f, "InterfaceCreationError: Invalid interface direction for fallback interface")
            }
            InterfaceCreateError::InterfaceOpenFailed => {
                core::write!(f, "InterfaceCreationError: Failed to open interface")
            }
            InterfaceCreateError::SetupDataParseError => {
                core::write!(f, "InterfaceCreationError: Setup data parse error")
            }
            InterfaceCreateError::InvalidSetupData(details) => {
                core::write!(f, "InterfaceCreationError: Invalid setup data - {}", details)
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
                core::write!(f, "ComHubError: Interface does not exit")
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
    SocketDisconnected,
    SocketUninitialized,
    SocketEndpointAlreadyRegistered,
}
