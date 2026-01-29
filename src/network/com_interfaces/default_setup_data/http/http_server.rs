use crate::{serde::Deserialize};
use core::prelude::rust_2024::*;
use core::time::Duration;
use serde::Serialize;
use crate::network::com_interfaces::com_interface::properties::{ComInterfaceProperties, InterfaceDirection};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct HTTPServerInterfaceSetupData {
    // TODO: address etc like TCP server setup data
    pub port: u16,
}

impl HTTPServerInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "http-server".to_string(),
            channel: "http".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            direction: InterfaceDirection::InOut,
            ..ComInterfaceProperties::default()
        }
    }
}