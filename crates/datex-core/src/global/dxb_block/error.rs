use strum::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum DxbHeaderParsingError {
    InsufficientLength,
    InvalidMagicNumber,
}

#[derive(Debug)]
pub enum DXBBlockParseError {
    IOError(String),
    ParseError(binrw::Error),
    IllegalState,
}

impl From<binrw::Error> for DXBBlockParseError {
    fn from(err: binrw::Error) -> Self {
        DXBBlockParseError::ParseError(err)
    }
}

impl From<String> for DXBBlockParseError {
    fn from(err: String) -> Self {
        DXBBlockParseError::IOError(err)
    }
}
