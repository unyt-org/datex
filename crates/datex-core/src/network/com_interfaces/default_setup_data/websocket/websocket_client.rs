use crate::prelude::*;
use core::time::Duration;
use serde::{Deserialize, Serialize};
use crate::network::com_interfaces::com_interface::properties::ComInterfaceProperties;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct WebSocketClientInterfaceSetupData {
    /// A websocket URL (ws:// or wss://).
    pub url: String,
}

impl WebSocketClientInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "websocket-client".to_string(),
            channel: "websocket".to_string(),
            round_trip_time: Duration::from_millis(40),
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
}