use super::serial_common::SerialInterfaceSetupData;
use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            error::ComInterfaceError,
            factory::{ComInterfaceSyncFactory},
            properties::{InterfaceDirection, ComInterfaceProperties},
            state::ComInterfaceState,
        },
    },
    std_sync::Mutex,
    stdlib::{sync::Arc, time::Duration},
    task::{spawn, spawn_blocking},
};
use core::{prelude::rust_2024::*, result::Result};
use log::{error, warn};
use datex_core::network::com_interfaces::com_interface::factory::ComInterfaceConfiguration;
use crate::global::dxb_block::DXBBlock;
use crate::network::com_interfaces::com_interface::factory::{SocketConfiguration, SendCallback, SendFailure, SocketProperties, SendSuccess};

impl SerialInterfaceSetupData {
    const TIMEOUT: Duration = Duration::from_millis(1000);
    const BUFFER_SIZE: usize = 1024;
    const DEFAULT_BAUD_RATE: u32 = 115200;

    pub fn get_available_ports() -> Vec<String> {
        serialport::available_ports()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|port| port.port_name.into())
            .collect()
    }

    fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let port_name = self.port_name.clone().ok_or(
            ComInterfaceCreateError::invalid_setup_data("Port name is required"),
        )?;

        if port_name.is_empty() {
            return Err(ComInterfaceCreateError::InvalidSetupData(
                "Port name cannot be empty".to_string(),
            ));
        }

        let port = serialport::new(port_name.clone(), self.baud_rate)
            .timeout(Self::TIMEOUT)
            .open()
            .map_err(|err| {
                ComInterfaceError::connection_error_with_details(err)
            })?;
        let port = Arc::new(Mutex::new(port));
        let port_clone = port.clone();

        Ok(ComInterfaceConfiguration::new_single_socket(
            ComInterfaceProperties {
                name: Some(port_name),
                ..Self::get_default_properties()
            },
            SocketConfiguration::new(
                SocketProperties::new(InterfaceDirection::InOut, 1),
                async gen move {
                    loop {
                        let result = spawn_blocking({
                            let port = port_clone.clone();
                            move || {
                                let mut buffer = [0u8; Self::BUFFER_SIZE];
                                match port.try_lock().unwrap().read(&mut buffer) {
                                    Ok(n) if n > 0 => Some(buffer[..n].to_vec()),
                                    _ => None,
                                }
                            }
                        }).await;
                        match result {
                            Ok(Some(incoming)) => {
                                yield Ok(incoming);
                            }
                            _ => {
                                error!("Serial read error or shutdown");
                                return yield Err(());
                            }
                        }
                    }
                },
                SendCallback::new_sync(
                    move |block: DXBBlock|
                        port.lock()
                            .unwrap()
                            .write_all(block.to_bytes().as_slice())
                            .map_err(|e| {
                                error!("Serial write error: {e}");
                                SendFailure(block)
                            })
                            .map(|_| SendSuccess::Sent)
                )
            )
        ))
    }
}

impl ComInterfaceSyncFactory for SerialInterfaceSetupData {
    fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        self.create_interface()
    }

    fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "serial".to_string(),
            channel: "serial".to_string(),
            round_trip_time: Duration::from_millis(40),
            max_bandwidth: 100,
            ..ComInterfaceProperties::default()
        }
    }
}
