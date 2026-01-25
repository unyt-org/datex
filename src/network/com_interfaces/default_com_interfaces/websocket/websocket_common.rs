use crate::stdlib::string::String;
use crate::stdlib::vec::Vec;
use core::{fmt::Display, prelude::rust_2024::*, result::Result};
use serde::{Deserialize, Serialize};
use strum::Display;
use thiserror::Error;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct WebSocketClientInterfaceSetupData {
    /// A websocket URL (ws:// or wss://).
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub enum TLSMode {
    /// The TLS certificate is handled externally (e.g., by a reverse proxy or load balancer).
    HandledExternally,
    /// The server must handle TLS using the provided certificate.
    WithCertificate {
        private_key: Vec<u8>,
        certificate: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct WebSocketServerInterfaceSetupData {
    /// The address to bind the WebSocket server to (e.g., "0.0.0.0:8080").
    pub bind_address: String,
    /// A list of addresses the server should accept connections from,
    /// along with their optional TLS mode.
    /// E.g., [("example.com", Some(TLSMode::WithCertificate { ... })), ("example.org:1234", None)]
    pub accept_addresses: Option<Vec<(String, Option<TLSMode>)>>,
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
