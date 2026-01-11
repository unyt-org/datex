use super::tcp_common::TCPServerInterfaceSetupData;
use crate::core::net::AddrParseError;
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
use crate::network::com_interfaces::com_interface::{
    ComInterface, ComInterfaceImplEvent,
};
use crate::std_sync::Mutex;
use crate::stdlib::collections::HashMap;
use crate::stdlib::net::SocketAddr;
use crate::stdlib::rc::Rc;
use crate::stdlib::sync::Arc;
use crate::task::{UnboundedReceiver, UnboundedSender};
use crate::task::{create_unbounded_channel, spawn_with_panic_notify_default};
use core::prelude::rust_2024::*;
use core::result::Result;
use core::time::Duration;
use log::{error, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::Notify;

pub struct TCPServerNativeInterface {
    // TODO: properties not really needed, just for testing purposes, can we remove this?
    pub address: SocketAddr,
    com_interface: Rc<ComInterface>,
    pub tx_by_socket:
        Arc<Mutex<HashMap<ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>>>>,
}

impl TCPServerNativeInterface {
    async fn create(
        setup_data: TCPServerInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
        let host = setup_data
            .host
            .clone()
            .unwrap_or_else(|| "0.0.0.0".to_string());

        let address: SocketAddr = format!("{}:{}", host, setup_data.port)
            .parse()
            .map_err(|e: AddrParseError| {
                InterfaceCreateError::InvalidSetupData(e.to_string())
            })?;

        let listener = TcpListener::bind(address).await.map_err(|e| {
            InterfaceCreateError::InterfaceError(
                ComInterfaceError::connection_error_with_details(e),
            )
        })?;
        info!("TCP Server listening on {address}");

        let tx_by_socket = Arc::new(Mutex::new(HashMap::new()));
        let tx_by_socket_clone = tx_by_socket.clone();

        let manager = com_interface.socket_manager();
        spawn_with_panic_notify_default(async move {
            loop {
                // Accept an incoming connection
                match listener.accept().await {
                    Ok((stream, _)) => {
                        // Initialize socket in com socket manager
                        let (socket_uuid, rx_sender) =
                            manager.lock().unwrap().create_and_init_socket(
                                InterfaceDirection::InOut,
                                1,
                            );
                        // Handle the client connection
                        let (tcp_read_half, tcp_write_half) =
                            stream.into_split();
                        let (tx_sender, tx_receiver) =
                            create_unbounded_channel::<Vec<u8>>();

                        // Spawn a task to handle outgoing messages to the client
                        spawn_with_panic_notify_default(async move {
                            Self::handle_send(tcp_write_half, tx_receiver).await
                        });

                        // Store the sender in the map
                        tx_by_socket_clone
                            .try_lock()
                            .unwrap()
                            .insert(socket_uuid, tx_sender);

                        // Spawn a task to handle incoming messages from the client
                        spawn_with_panic_notify_default(async move {
                            Self::handle_receive(tcp_read_half, rx_sender).await
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {e}");
                        continue;
                    }
                }
            }
        });

        spawn_with_panic_notify_default(Self::event_handler_task(
            com_interface.take_interface_impl_event_receiver(),
            tx_by_socket.clone(),
            Arc::new(Notify::new()),
        ));

        Ok((
            TCPServerNativeInterface {
                address,
                com_interface,
                tx_by_socket,
            },
            Self::get_default_properties(),
        ))
    }

    #[allow(clippy::await_holding_lock)]
    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        mut receiver: UnboundedReceiver<ComInterfaceImplEvent>,
        tx_by_socket: Arc<
            Mutex<HashMap<ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>>>,
        >,
        shutdown_signal: Arc<Notify>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceImplEvent::SendBlock(block, socket_uuid) => {
                    let tx = tx_by_socket
                        .lock()
                        .map(|guard| guard.get(&socket_uuid).cloned());
                    let Ok(tx) = tx else {
                        error!("Client is not connected: {}", socket_uuid);
                        continue;
                    };
                    let mut tx = tx.unwrap();
                    if tx.start_send(block).is_err() {
                        error!("Write failed for {}", socket_uuid);
                    }
                }
                ComInterfaceImplEvent::Destroy => {
                    shutdown_signal.notify_waiters();
                }
                _ => todo!(),
            }
        }
    }

    async fn handle_receive(
        mut rx: OwnedReadHalf,
        mut bytes_in_sender: UnboundedSender<Vec<u8>>,
    ) {
        let mut buffer = [0u8; 1024];
        loop {
            match rx.read(&mut buffer).await {
                Ok(0) => {
                    warn!("Connection closed by peer");
                    break;
                }
                Ok(n) => {
                    bytes_in_sender.start_send(buffer[..n].to_vec()).expect(
                        "Failed to send received data to ComInterfaceSocket",
                    );
                }
                Err(e) => {
                    error!("Failed to read from socket: {e}");
                    break;
                }
            }
        }
    }

    async fn handle_send(
        mut tcp_write_half: OwnedWriteHalf,
        mut tx_receiver: UnboundedReceiver<Vec<u8>>,
    ) {
        while let Some(block) = tx_receiver.next().await {
            if tcp_write_half.write_all(&block).await.is_err() {
                // FIXME error handling when write fails
                break;
            }
        }
    }
}

impl ComInterfaceAsyncFactory for TCPServerNativeInterface {
    type SetupData = TCPServerInterfaceSetupData;
    fn create(
        setup_data: Self::SetupData,
        com_interface: Rc<ComInterface>,
    ) -> ComInterfaceAsyncFactoryResult<Self> {
        Box::pin(async move {
            TCPServerNativeInterface::create(setup_data, com_interface).await
        })
    }

    fn get_default_properties() -> InterfaceProperties {
        InterfaceProperties {
            interface_type: "tcp-server".to_string(),
            channel: "tcp".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            ..InterfaceProperties::default()
        }
    }
}

impl ComInterfaceImplementation for TCPServerNativeInterface {}

#[cfg(test)]
mod tests {
    use core::{assert_matches::assert_matches, u16};

    use datex_macros::async_test;

    use crate::network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::{
            com_interface::ComInterface,
            default_com_interfaces::tcp::{
                tcp_common::TCPServerInterfaceSetupData,
                tcp_server_native_interface::TCPServerNativeInterface,
            },
        },
    };

    #[async_test]
    async fn test_construct() {
        const PORT: u16 = 5088;
        let com_interface =
            ComInterface::create_async_with_implementation::<
                TCPServerNativeInterface,
            >(TCPServerInterfaceSetupData::new_with_port(PORT))
            .await
            .unwrap();
        let tcp_server_interface =
            com_interface.implementation::<TCPServerNativeInterface>();
        assert_eq!(tcp_server_interface.address.port(), PORT);
    }

    #[async_test]
    async fn test_invalid_address() {
        assert_matches!(
            ComInterface::create_async_with_implementation::<
                TCPServerNativeInterface,
            >(TCPServerInterfaceSetupData::new_with_host_and_port(
                "invalid-address".to_string(),
                5088
            ))
            .await,
            Err(InterfaceCreateError::InvalidSetupData(_))
        );
    }
}
