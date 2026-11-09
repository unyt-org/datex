use crate::{
    network::com_interfaces::com_interface::properties::ComInterfaceProperties,
    prelude::*,
};
use datex_macros_internal::Datex;
use serde::{Deserialize, Serialize};
use crate::network::com_interfaces::com_interface::properties::ReconnectionConfig;

#[derive(Datex, Clone, Debug, Serialize, Deserialize)]
#[datex(structural)]
pub struct WebSocketClientInterfaceSetupData {
    /// A websocket URL (ws:// or wss://).
    pub url: String,
}

impl WebSocketClientInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "websocket-client".to_string(),
            channel: "websocket".to_string(),
            round_trip_time: 40,
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
    pub fn get_reconnection_config() -> ReconnectionConfig {
        ReconnectionConfig::reconnect()
    }
}
