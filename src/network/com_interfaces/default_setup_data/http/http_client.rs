use crate::stdlib::string::String;
use core::prelude::rust_2024::*;
use core::time::Duration;
use serde::{Deserialize, Serialize};
use crate::network::com_interfaces::com_interface::properties::ComInterfaceProperties;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct HTTPClientInterfaceSetupData {
    /// A websocket URL (http:// or https://).
    pub url: String,
}

impl HTTPClientInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "http-client".to_string(),
            channel: "http".to_string(),
            round_trip_time: Duration::from_millis(40),
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
}