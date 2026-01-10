use core::fmt::Display;
use core::fmt::Debug;

pub enum ComInterfaceError {
    SocketNotFound,
    SocketAlreadyExists,
    ConnectionError(Option<Box<dyn Display>>),
    SendError,
    ReceiveError,
}

impl PartialEq for ComInterfaceError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ComInterfaceError::SocketNotFound, ComInterfaceError::SocketNotFound) => true,
            (ComInterfaceError::SocketAlreadyExists, ComInterfaceError::SocketAlreadyExists) => true,
            (ComInterfaceError::ConnectionError(Some(x)), ComInterfaceError::ConnectionError(Some(y))) => {
                format!("{}", x) == format!("{}", y)
            },
            (ComInterfaceError::ConnectionError(None), ComInterfaceError::ConnectionError(None)) => true,
            (ComInterfaceError::SendError, ComInterfaceError::SendError) => true,
            (ComInterfaceError::ReceiveError, ComInterfaceError::ReceiveError) => true,
            _ => false,
        }
    }
}

impl Debug for ComInterfaceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ComInterfaceError::SocketNotFound => write!(f, "SocketNotFound"),
            ComInterfaceError::SocketAlreadyExists => write!(f, "SocketAlreadyExists"),
            ComInterfaceError::ConnectionError(Some(details)) => {
                write!(f, "ConnectionError: {}", details)
            }
            ComInterfaceError::ConnectionError(None) => write!(f, "ConnectionError"),
            ComInterfaceError::SendError => write!(f, "SendError"),
            ComInterfaceError::ReceiveError => write!(f, "ReceiveError"),
        }
    }
}

impl ComInterfaceError {
    pub fn connection_error_with_details<T: Display + 'static>(details: T) -> Self {
        ComInterfaceError::ConnectionError(Some(Box::new(details)))
    }
    pub fn connection_error() -> Self {
        ComInterfaceError::ConnectionError(None)
    }
}