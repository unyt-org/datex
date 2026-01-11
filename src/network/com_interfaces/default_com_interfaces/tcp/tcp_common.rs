use crate::stdlib::string::String;
use core::prelude::rust_2024::*;
use serde::{Deserialize, Serialize};
use strum::Display;
use thiserror::Error;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct TCPClientInterfaceSetupData {
    pub address: String,
}

#[derive(Serialize, Deserialize, Default)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct TCPServerInterfaceSetupData {
    pub port: u16,
    pub host: Option<String>,
}
impl TCPServerInterfaceSetupData {
    pub fn new_with_port(port: u16) -> Self {
        TCPServerInterfaceSetupData { port, host: None }
    }
    pub fn new_with_host_and_port(host: String, port: u16) -> Self {
        TCPServerInterfaceSetupData {
            port,
            host: Some(host),
        }
    }
}

#[derive(Debug, Display, Error, Clone, PartialEq)]
pub enum TCPError {
    Other(String),
    InvalidAddress,
    ConnectionError,
    SendError,
    ReceiveError,
}
