use core::time::Duration;
use serde::Serialize;
use crate::network::com_interfaces::com_interface::properties::ComInterfaceProperties;
use super::tcp_client::TCPClientInterfaceSetupData;
use crate::serde::Deserialize;

#[derive(Serialize, Deserialize, Default)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct TCPServerInterfaceSetupData {
    pub port: u16,
    pub host: Option<String>,
}
impl TCPServerInterfaceSetupData {
    pub fn new_with_port(port: u16) -> Self {
        TCPServerInterfaceSetupData { port, host: None }
    }
    pub fn new_with_host_and_port(host: String, port: u16) -> Self {
        TCPServerInterfaceSetupData {
            port,
            host: Some(host),
        }
    }
    
    pub fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "tcp-server".to_string(),
            channel: "tcp".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
    
    pub fn get_clients_setup_data(&self) -> Vec<TCPClientInterfaceSetupData> {
        todo!()
    }
}