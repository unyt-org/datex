use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::{
            com_interface::properties::{
                ComInterfaceProperties, InterfaceDirection,
            },
            default_setup_data::{
                http::http_client::HTTPClientInterfaceSetupData,
                http_common::{AcceptAddress, get_clients_setup_data},
            },
        },
    },
    prelude::*,
    runtime::RuntimeConfigInterface,
    serde::Deserialize,
};
use core::time::Duration;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct HTTPServerInterfaceSetupData {
    /// The address to bind the HTTP server to (e.g., "0.0.0.0:8080").
    pub bind_address: String,
    /// A list of addresses the server should accept connections from,
    /// along with their optional TLS mode.
    /// E.g., [("example.com", Some(TLSMode::WithCertificate { ... })), ("example.org:1234", None)]
    pub accept_addresses: Option<Vec<AcceptAddress>>,
}

impl HTTPServerInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "http-server".to_string(),
            channel: "http".to_string(),
            round_trip_time: Duration::from_millis(40),
            max_bandwidth: 1000,
            direction: InterfaceDirection::InOut,
            continuous_connection: false,
            allow_redirects: false,
            ..ComInterfaceProperties::default()
        }
    }

    /// Generates the setup data for HTTP client interfaces based on the server's accept addresses.
    pub fn get_clients_setup_data(
        accept_addresses: Option<Vec<AcceptAddress>>,
    ) -> Result<Option<Vec<RuntimeConfigInterface>>, ComInterfaceCreateError>
    {
        get_clients_setup_data(
            accept_addresses,
            ("http".to_string(), "https".to_string()),
            "http-client".to_string(),
            |url| HTTPClientInterfaceSetupData { url },
        )
    }
}
