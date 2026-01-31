use core::time::Duration;
use serde::Serialize;
use crate::network::com_hub::errors::ComInterfaceCreateError;
use crate::network::com_interfaces::com_interface::properties::ComInterfaceProperties;
use crate::network::com_interfaces::default_setup_data::http_common::{get_clients_setup_data, AcceptAddresses};
use super::websocket_client::WebSocketClientInterfaceSetupData;
use crate::runtime::RuntimeConfigInterface;
use crate::serde::Deserialize;


#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct WebSocketServerInterfaceSetupData {
    /// The address to bind the WebSocket server to (e.g., "0.0.0.0:8080").
    pub bind_address: String,
    /// A list of addresses the server should accept connections from,
    /// along with their optional TLS mode.
    /// E.g., [("example.com", Some(TLSMode::WithCertificate { ... })), ("example.org:1234", None)]
    pub accept_addresses: Option<AcceptAddresses>,
}

impl WebSocketServerInterfaceSetupData {
    /// Returns the default properties for a WebSocket server interface
    pub(crate) fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "websocket-server".to_string(),
            channel: "websocket".to_string(),
            round_trip_time: Duration::from_millis(40),
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }

    /// Generates the setup data for WebSocket client interfaces based on the server's accept addresses.
    pub fn get_clients_setup_data(accept_addresses: Option<AcceptAddresses>) -> Result<Option<Vec<RuntimeConfigInterface>>, ComInterfaceCreateError> {
        get_clients_setup_data(
            accept_addresses,
            ("ws".to_string(), "wss".to_string()),
            "websocket-client".to_string(),
            |url| WebSocketClientInterfaceSetupData { url },
        )
    }
}