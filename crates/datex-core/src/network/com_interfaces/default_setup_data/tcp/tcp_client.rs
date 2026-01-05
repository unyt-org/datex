use crate::prelude::*;
use core::time::Duration;
use serde::Serialize;
use crate::network::com_interfaces::com_interface::properties::ComInterfaceProperties;
use crate::serde::Deserialize;
use crate::prelude::*;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct TCPClientInterfaceSetupData {
    pub address: String,
}

impl TCPClientInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "tcp-client".to_string(),
            channel: "tcp".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
}