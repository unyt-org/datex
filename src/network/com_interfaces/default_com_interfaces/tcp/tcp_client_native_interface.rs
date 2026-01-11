use super::tcp_common::TCPClientInterfaceSetupData;

use crate::network::com_hub::errors::InterfaceCreateError;
use crate::network::com_interfaces::com_interface::error::ComInterfaceError;
use crate::network::com_interfaces::com_interface::implementation::{
    ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
    ComInterfaceImplementation, ComInterfaceSyncFactory,
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
use crate::stdlib::net::SocketAddr;
use crate::stdlib::rc::Rc;
use crate::stdlib::sync::Arc;
use crate::task::{
    UnboundedReceiver, UnboundedSender, spawn, spawn_with_panic_notify_default,
};
use core::prelude::rust_2024::*;
use core::result::Result;
use core::str::FromStr;
use core::time::Duration;
use log::{error, warn};
use std::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::select;
use tokio::sync::Notify;

pub struct TCPClientNativeInterface {
    pub address: SocketAddr,
    pub socket_uuid: ComInterfaceSocketUUID,
    com_interface: Rc<ComInterface>,
}

impl TCPClientNativeInterface {
    async fn create(
        setup_data: TCPClientInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
        let address = SocketAddr::from_str(&setup_data.address)
            .map_err(InterfaceCreateError::invalid_setup_data)?;

        let stream = TcpStream::connect(address).await.map_err(|error| {
            ComInterfaceError::connection_error_with_details(error)
        })?;

        let (read_half, tx) = stream.into_split();

        let (socket_uuid, sender) = com_interface
            .socket_manager()
            .lock()
            .unwrap()
            .create_and_init_socket(InterfaceDirection::InOut, 1);

        let state = com_interface.state();
        let shutdown_signal = Arc::new(Notify::new());
        let shutdown_signal_clone = shutdown_signal.clone();

        spawn(async move {
            Self::handle_receive(
                read_half,
                sender,
                state,
                shutdown_signal_clone,
            )
            .await;
        });

        spawn_with_panic_notify_default(Self::event_handler_task(
            tx,
            com_interface.take_interface_impl_event_receiver(),
        ));

        Ok((
            TCPClientNativeInterface {
                address,
                socket_uuid,
                com_interface,
            },
            InterfaceProperties {
                name: Some(setup_data.address),
                ..Self::get_default_properties()
            },
        ))
    }

    /// Background task to handle incoming messages
    async fn handle_receive(
        read_half: tokio::net::tcp::OwnedReadHalf,
        mut sender: UnboundedSender<Vec<u8>>,
        state: Arc<Mutex<ComInterfaceStateWrapper>>,
        shutdown_signal_clone: Arc<Notify>,
    ) {
        let mut reader = read_half;
        let mut buffer = [0u8; 1024];
        loop {
            select! {
                next = reader.read(&mut buffer) => {
                    match next {
                        Ok(0) => {
                            warn!("Connection closed by peer");
                            state.lock().unwrap().set(ComInterfaceState::Destroyed);
                            break;
                        }
                        Ok(n) => {
                            sender.start_send(buffer[..n].to_vec()).unwrap();
                        }
                        Err(e) => {
                            error!("Failed to read from socket: {e}");
                            state
                                .try_lock()
                                .unwrap()
                                .set(ComInterfaceState::Destroyed);
                            break;
                        }
                    }
                }
                _ = shutdown_signal_clone.notified() => {
                    break;
                }
            }
        }
    }

    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        mut write: OwnedWriteHalf,
        mut receiver: UnboundedReceiver<ComInterfaceImplEvent>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceImplEvent::SendBlock(block, _) => {
                    if let Err(e) = write.write_all(&block).await {
                        error!("Failed to send data: {}", e);
                        // TODO: handle error properly
                    }
                }
                _ => todo!(),
            }
        }
    }
}

impl ComInterfaceImplementation for TCPClientNativeInterface {}

impl ComInterfaceAsyncFactory for TCPClientNativeInterface {
    type SetupData = TCPClientInterfaceSetupData;

    fn create(
        setup_data: Self::SetupData,
        com_interface: Rc<ComInterface>,
    ) -> ComInterfaceAsyncFactoryResult<Self> {
        Box::pin(async move {
            TCPClientNativeInterface::create(setup_data, com_interface).await
        })
    }

    fn get_default_properties() -> InterfaceProperties {
        InterfaceProperties {
            interface_type: "tcp-client".to_string(),
            channel: "tcp".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            ..InterfaceProperties::default()
        }
    }
}
