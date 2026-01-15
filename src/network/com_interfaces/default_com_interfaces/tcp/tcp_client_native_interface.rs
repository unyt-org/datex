use super::tcp_common::TCPClientInterfaceSetupData;

use crate::{
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterface, ComInterfaceEvent,
            error::ComInterfaceError,
            implementation::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::{InterfaceDirection, InterfaceProperties},
            socket::ComInterfaceSocketUUID,
            state::{ComInterfaceState, ComInterfaceStateWrapper},
        },
    },
    stdlib::{net::SocketAddr, sync::Arc},
    task::{
        UnboundedReceiver, UnboundedSender, spawn_with_panic_notify_default,
    },
};
use core::{
    prelude::rust_2024::*, result::Result, str::FromStr, time::Duration,
};
use log::{error, warn};
use std::sync::Mutex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp::OwnedWriteHalf},
    select,
    sync::Notify,
};
use crate::network::com_interfaces::com_interface::ComInterfaceProxy;

pub struct TCPClientNativeInterface {
    pub address: SocketAddr,
    pub socket_uuid: ComInterfaceSocketUUID,
    com_interface: ComInterface,
}

/// Implementation of the TCP Client Native Interface
impl TCPClientInterfaceSetupData {
    async fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> Result<InterfaceProperties, InterfaceCreateError> {
        let address = SocketAddr::from_str(&self.address)
            .map_err(InterfaceCreateError::invalid_setup_data)?;

        let stream = TcpStream::connect(address).await.map_err(|error| {
            ComInterfaceError::connection_error_with_details(error)
        })?;

        let (read_half, tx) = stream.into_split();

        let (_, sender) = com_interface_proxy
            .create_and_init_socket(InterfaceDirection::InOut, 1);

        let shutdown_signal = com_interface_proxy.shutdown_signal();

        spawn_with_panic_notify_default(async move {
            Self::handle_receive(read_half, sender, com_interface_proxy.state, shutdown_signal)
                .await;
        });

        spawn_with_panic_notify_default(Self::event_handler_task(
            tx,
            com_interface_proxy.event_receiver,
        ));

        Ok(InterfaceProperties {
            name: Some(self.address),
            ..Self::get_default_properties()
        })
    }

    /// Background task to handle incoming messages
    async fn handle_receive(
        read_half: tokio::net::tcp::OwnedReadHalf,
        mut sender: UnboundedSender<Vec<u8>>,
        state: Arc<Mutex<ComInterfaceStateWrapper>>,
        shutdown_signal: Arc<Notify>,
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
                _ = shutdown_signal.notified() => {
                    break;
                }
            }
        }
    }

    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        mut write: OwnedWriteHalf,
        mut receiver: UnboundedReceiver<ComInterfaceEvent>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceEvent::SendBlock(block, _) => {
                    if let Err(e) = write.write_all(&block.to_bytes()).await {
                        error!("Failed to send data: {}", e);
                        // TODO: handle error properly
                    }
                }
                ComInterfaceEvent::Destroy => {
                    break;
                }
                _ => todo!(),
            }
        }
    }
}

impl ComInterfaceAsyncFactory for TCPClientInterfaceSetupData {
    fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> ComInterfaceAsyncFactoryResult {
        Box::pin(async move {
            self.create_interface(com_interface_proxy).await
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
