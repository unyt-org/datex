use crate::stdlib::string::String;
use core::prelude::rust_2024::*;
use serde::{Deserialize, Serialize};
use strum::Display;
use thiserror::Error;

#[derive(Serialize, Deserialize)]
pub struct SerialInterfaceSetupData {
    pub port_name: Option<String>,
    pub baud_rate: u32,
}

#[derive(Debug, Display, Error)]
pub enum SerialError {
    Other(String),
    PermissionError,
    PortNotFound,
    ConnectionError,
    SendError,
    ReceiveError,
}
