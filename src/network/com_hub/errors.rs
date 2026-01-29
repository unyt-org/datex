use core::fmt::Display;
use std::fmt::Debug;
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

pub enum ComInterfaceCreateError {
    ConnectionError(Option<Box<dyn Display>>),
    InterfaceCreationRequiresAsyncContext,
    InterfaceTypeNotRegistered(String),
    SetupDataParseError,
    InvalidSetupData(String),
    InterfaceAddError(InterfaceAddError),
}


impl From<InterfaceAddError> for ComInterfaceCreateError {
    fn from(err: InterfaceAddError) -> Self {
        ComInterfaceCreateError::InterfaceAddError(err)
    }
}

impl ComInterfaceCreateError {
    pub fn invalid_setup_data<T: Display>(details: T) -> Self {
        ComInterfaceCreateError::InvalidSetupData(details.to_string())
    }

    pub fn connection_error_with_details<T: Display + 'static>(
        details: T,
    ) -> Self {
        ComInterfaceCreateError::ConnectionError(Some(Box::new(details)))
    }
    pub fn connection_error() -> Self {
        ComInterfaceCreateError::ConnectionError(None)
    }
}

impl Debug for ComInterfaceCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ComInterfaceCreateError::InterfaceCreationRequiresAsyncContext => {
                write!(
                    f,
                    "ComInterfaceCreateError::InterfaceCreationRequiresAsyncContext"
                )
            }
            ComInterfaceCreateError::InterfaceTypeNotRegistered(ty) => {
                write!(
                    f,
                    "ComInterfaceCreateError::InterfaceTypeNotRegistered({})",
                    ty
                )
            }
            ComInterfaceCreateError::SetupDataParseError => {
                write!(f, "ComInterfaceCreateError::SetupDataParseError")
            }
            ComInterfaceCreateError::InvalidSetupData(details) => {
                write!(
                    f,
                    "ComInterfaceCreateError::InvalidSetupData({})",
                    details
                )
            }
            ComInterfaceCreateError::InterfaceAddError(add_err) => {
                write!(
                    f,
                    "ComInterfaceCreateError::InterfaceAddError({:?})",
                    add_err
                )
            }
            ComInterfaceCreateError::ConnectionError(Some(details)) => {
                write!(
                    f,
                    "ComInterfaceCreateError::ConnectionError({})",
                    details
                )
            }
            ComInterfaceCreateError::ConnectionError(None) => {
                write!(f, "ComInterfaceCreateError::ConnectionError(None)")
            }
        }
    }
}

impl Display for ComInterfaceCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ComInterfaceCreateError::InterfaceCreationRequiresAsyncContext => {
                write!(
                    f,
                    "ComInterfaceCreateError: Interface creation requires async context"
                )
            }
            ComInterfaceCreateError::InterfaceTypeNotRegistered(ty) => {
                write!(
                    f,
                    "ComInterfaceCreateError: Interface type '{}' is not registered",
                    ty
                )
            }
            ComInterfaceCreateError::SetupDataParseError => {
                write!(
                    f,
                    "ComInterfaceCreateError: Setup data parse error"
                )
            }
            ComInterfaceCreateError::InvalidSetupData(details) => {
                write!(
                    f,
                    "ComInterfaceCreateError: Invalid setup data - {}",
                    details
                )
            }
            ComInterfaceCreateError::InterfaceAddError(add_err) => {
                write!(
                    f,
                    "InterfaceCreationError: InterfaceAddError - {}",
                    add_err
                )
            }
            ComInterfaceCreateError::ConnectionError(Some(details)) => {
                write!(f, "ComInterfaceCreateError: Connection error: {}", details)
            }
            ComInterfaceCreateError::ConnectionError(None) => {
                write!(f, "ComInterfaceCreateError: Connection error")
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
