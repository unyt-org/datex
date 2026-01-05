use crate::prelude::*;
use core::time::Duration;
use serde::{Deserialize, Serialize};
use crate::network::com_interfaces::com_interface::properties::ComInterfaceProperties;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct SerialClientInterfaceSetupData {
    pub port_name: Option<String>,
    pub baud_rate: u32,
}

impl SerialClientInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "serial".to_string(),
            channel: "serial".to_string(),
            round_trip_time: Duration::from_millis(40),
            max_bandwidth: 100,
            ..ComInterfaceProperties::default()
        }
    }
}