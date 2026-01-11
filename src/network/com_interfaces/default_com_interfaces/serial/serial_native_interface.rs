use super::serial_common::SerialInterfaceSetupData;
use crate::network::com_hub::errors::InterfaceCreateError;
use crate::network::com_interfaces::com_interface::error::ComInterfaceError;
use crate::network::com_interfaces::com_interface::implementation::ComInterfaceImplementation;
use crate::network::com_interfaces::com_interface::implementation::{
    ComInterfaceAsyncFactory, ComInterfaceSyncFactory,
};
use crate::network::com_interfaces::com_interface::properties::{
    InterfaceDirection, InterfaceProperties,
};
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;
use crate::network::com_interfaces::com_interface::state::{
    ComInterfaceState, ComInterfaceStateWrapper,
};
use crate::network::com_interfaces::com_interface::{
    ComInterface, ComInterfaceImplEvent,
};
use crate::std_sync::Mutex;
use crate::stdlib::rc::Rc;
use crate::stdlib::{future::Future, pin::Pin, sync::Arc, time::Duration};
use crate::task::{UnboundedReceiver, spawn_with_panic_notify_default};
use crate::{task::spawn, task::spawn_blocking};
use core::prelude::rust_2024::*;
use core::result::Result;
use futures_util::stream::SplitSink;
use log::{debug, error, warn};
use serialport::SerialPort;
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

pub struct SerialNativeInterface {
    com_interface: Rc<ComInterface>,
    pub socket_uuid: ComInterfaceSocketUUID,
}

impl SerialNativeInterface {
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

    fn create(
        setup_data: SerialInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
        let state = com_interface.state();

        let port_name = setup_data.port_name.clone().ok_or(
            InterfaceCreateError::invalid_setup_data("Port name is required"),
        )?;

        if port_name.is_empty() {
            return Err(InterfaceCreateError::InvalidSetupData(
                "Port name cannot be empty".to_string(),
            ));
        }

        let port = serialport::new(port_name, setup_data.baud_rate)
            .timeout(Self::TIMEOUT)
            .open()
            .map_err(|err| {
                ComInterfaceError::connection_error_with_details(err)
            })?;
        let port = Arc::new(Mutex::new(port));
        let port_clone = port.clone();

        let (socket_uuid, mut sender) = com_interface
            .socket_manager()
            .lock()
            .unwrap()
            .create_and_init_socket(InterfaceDirection::InOut, 1);

        let shutdown_signal = Arc::new(Notify::new());
        let shutdown_signal_clone = shutdown_signal.clone();
        spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_signal_clone.notified() => {
                        warn!("Shutting down serial task...");
                        break;
                    },
                    result = spawn_blocking({
                        let port = port_clone.clone();
                        move || {
                            let mut buffer = [0u8; Self::BUFFER_SIZE];
                            match port.try_lock().unwrap().read(&mut buffer) {
                                Ok(n) if n > 0 => Some(buffer[..n].to_vec()),
                                _ => None,
                            }
                        }
                    }) => {
                        match result {
                            Ok(Some(incoming)) => {
                                let size = incoming.len();
                                sender.start_send(incoming).unwrap();
                                debug!(
                                    "Received data from serial port: {size}"
                                );
                            }
                            _ => {
                                error!("Serial read error or shutdown");
                                break;
                            }
                        }
                    }
                }
            }
            // FIXME #212 add reconnect logic (close gracefully and reopen)
            state.try_lock().unwrap().set(ComInterfaceState::Destroyed);
            warn!("Serial socket closed");
        });

        spawn_with_panic_notify_default(Self::event_handler_task(
            com_interface.take_interface_impl_event_receiver(),
            port.clone(),
            shutdown_signal.clone(),
        ));

        Ok((
            SerialNativeInterface {
                com_interface,
                socket_uuid,
            },
            Self::get_default_properties(),
        ))
    }

    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        mut receiver: UnboundedReceiver<ComInterfaceImplEvent>,
        mut port: Arc<Mutex<Box<dyn SerialPort>>>,
        shutdown_signal: Arc<Notify>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceImplEvent::SendBlock(block, _) => {
                    port.lock().unwrap().write_all(block.as_slice()).unwrap();
                }
                ComInterfaceImplEvent::Destroy => {
                    shutdown_signal.notify_waiters();
                    break;
                }
                _ => todo!(),
            }
        }
    }
}

impl ComInterfaceSyncFactory for SerialNativeInterface {
    type SetupData = SerialInterfaceSetupData;
    fn create(
        setup_data: Self::SetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
        SerialNativeInterface::create(setup_data, com_interface)
    }

    fn get_default_properties() -> InterfaceProperties {
        InterfaceProperties {
            interface_type: "serial".to_string(),
            channel: "serial".to_string(),
            round_trip_time: Duration::from_millis(40),
            max_bandwidth: 100,
            ..InterfaceProperties::default()
        }
    }
}

impl ComInterfaceImplementation for SerialNativeInterface {}
