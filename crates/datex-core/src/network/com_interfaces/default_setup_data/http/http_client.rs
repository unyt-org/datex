use crate::{
    network::com_interfaces::com_interface::properties::ComInterfaceProperties,
    prelude::*,
};
use datex_macros_internal::Datex;
use serde::{Deserialize, Serialize};

#[derive(Datex, Debug, Serialize, Deserialize)]
#[datex(only_structural)]
pub struct HTTPClientInterfaceSetupData {
    /// A websocket URL (http:// or https://).
    pub url: String,
}

impl HTTPClientInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "http-client".to_string(),
            channel: "http".to_string(),
            round_trip_time: 40,
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
}
