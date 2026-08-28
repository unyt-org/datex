use crate::prelude::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Number(NumberParseError),
    Time(TimeError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberParseError {
    InvalidFormat,
    OutOfRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeError {
    ParseError(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Number(e) => write!(f, "{}", e),
            Error::Time(e) => write!(f, "{}", e),
        }
    }
}

impl core::fmt::Display for NumberParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NumberParseError::OutOfRange => {
                write!(f, "The number is out of range for the specified type.")
            }
            NumberParseError::InvalidFormat => {
                write!(f, "The number format is invalid.")
            }
        }
    }
}

impl core::fmt::Display for TimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TimeError::ParseError(e) => write!(f, "{}", e),
        }
    }
}

impl TimeError {
    #[inline]
    pub fn parse(msg: impl Into<String>) -> Self {
        TimeError::ParseError(msg.into())
    }
}
