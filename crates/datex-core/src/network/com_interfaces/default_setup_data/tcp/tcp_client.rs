use crate::{
    network::com_interfaces::com_interface::properties::ComInterfaceProperties,
    prelude::*,
};
use datex_macros_internal::Datex;
use serde::{Deserialize, Serialize};

#[derive(Datex, Serialize, Deserialize)]
#[datex(structural_recursive)]
pub struct TCPClientInterfaceSetupData {
    pub address: String,
}

impl TCPClientInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "tcp-client".to_string(),
            channel: "tcp".to_string(),
            round_trip_time: 20,
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
}
