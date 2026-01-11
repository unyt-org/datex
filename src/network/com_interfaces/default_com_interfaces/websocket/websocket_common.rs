use crate::stdlib::string::String;
use core::{fmt::Display, prelude::rust_2024::*, result::Result};
use serde::{Deserialize, Serialize};
use strum::Display;
use thiserror::Error;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct WebSocketClientInterfaceSetupData {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct WebSocketServerInterfaceSetupData {
    pub port: u16,
    /// if true, the server will use wss (secure WebSocket). Defaults to true.
    pub secure: Option<bool>,
}

#[derive(Debug)]
pub enum URLError {
    InvalidURL,
    InvalidScheme,
}
impl Display for URLError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            URLError::InvalidURL => core::write!(f, "URLError: Invalid URL"),
            URLError::InvalidScheme => {
                core::write!(f, "URLError: Invalid URL scheme")
            }
        }
    }
}

#[derive(Debug, Display, Error, Clone, PartialEq)]
pub enum WebSocketError {
    Other(String),
    InvalidURL,
    ConnectionError,
    SendError,
    ReceiveError,
}

#[derive(Debug, Display, Error, Clone, PartialEq)]
pub enum WebSocketServerError {
    WebSocketError(WebSocketError),
    InvalidPort,
}

/// Parses a WebSocket URL and returns a `Url` object.
/// If no protocol is specified, it defaults to `ws` or `wss` based on the `secure` parameter.
pub fn parse_url(address: &str) -> Result<Url, URLError> {
    let mut url = Url::parse(address).map_err(|_| URLError::InvalidURL)?;
    match url.scheme() {
        "https" => url.set_scheme("wss").unwrap(),
        "http" => url.set_scheme("ws").unwrap(),
        "wss" | "ws" => (),
        _ => return Err(URLError::InvalidScheme),
    }
    Ok(url)
}
