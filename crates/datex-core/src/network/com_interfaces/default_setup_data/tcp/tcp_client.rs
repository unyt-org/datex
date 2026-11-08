use crate::{
    network::com_interfaces::com_interface::properties::ComInterfaceProperties,
    prelude::*,
};
use datex_macros_internal::Datex;
use serde::{Deserialize, Serialize};
use crate::network::com_interfaces::com_interface::properties::ReconnectionConfig;

#[derive(Datex, Clone, Serialize, Deserialize)]
#[datex(structural)]
pub struct TCPClientInterfaceSetupData {
    pub address: String,
}

impl TCPClientInterfaceSetupData {
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "tcp-client".to_string(),
            channel: "tcp".to_string(),
            reconnection_config: ReconnectionConfig::reconnect(),
            round_trip_time: 20,
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
}
