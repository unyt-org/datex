use super::tcp_common::TCPServerInterfaceSetupData;
use crate::{
    channel::{
        mpmc::BroadcastReceiver,
        mpsc::{UnboundedReceiver, UnboundedSender, create_unbounded_channel},
    },
    core::net::AddrParseError,
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterfaceEvent, ComInterfaceProxy,
            error::ComInterfaceError,
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::{InterfaceDirection, InterfaceProperties},
            socket::ComInterfaceSocketUUID,
        },
    },
    std_sync::Mutex,
    stdlib::{collections::HashMap, net::SocketAddr, sync::Arc},
    task::spawn_with_panic_notify_default,
};
use async_select::select;
use core::{prelude::rust_2024::*, result::Result, time::Duration};
use log::{error, info, warn};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpListener,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};

impl TCPServerInterfaceSetupData {
    async fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> Result<InterfaceProperties, InterfaceCreateError> {
        let host = self.host.clone().unwrap_or_else(|| "0.0.0.0".to_string());

        let address: SocketAddr = format!("{}:{}", host, self.port)
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

        let mut shutdown_signal = com_interface_proxy.shutdown_receiver();
        let manager = com_interface_proxy.socket_manager;
        spawn_with_panic_notify_default(async move {
            loop {
                select! {
                    // Wait for an incoming connection
                    accept_result = listener.accept() => {
                        let shutdown_signal_clone = shutdown_signal.clone();
                        match accept_result {
                            Ok((stream, _)) => {
                                // Initialize socket in com socket manager
                                let (socket_uuid, rx_sender) =
                                    manager.lock().unwrap().create_and_init_socket_with_optional_endpoint(
                                        InterfaceDirection::InOut,
                                        1,
                                        None
                                    );
                                // Handle the client connection
                                let (tcp_read_half, tcp_write_half) =
                                    stream.into_split();
                                let (tx_sender, tx_receiver) =
                                    create_unbounded_channel::<Vec<u8>>();

                                let shutdown_signal = shutdown_signal_clone.clone();
                                // Spawn a task to handle outgoing messages to the client
                                spawn_with_panic_notify_default(async move {
                                    Self::handle_send(tcp_write_half, tx_receiver, shutdown_signal).await
                                });

                                // Store the sender in the map
                                tx_by_socket_clone
                                    .try_lock()
                                    .unwrap()
                                    .insert(socket_uuid, tx_sender);

                                // Spawn a task to handle incoming messages from the client
                                spawn_with_panic_notify_default(async move {
                                    Self::handle_receive(tcp_read_half, rx_sender, shutdown_signal_clone).await
                                });
                            }
                            Err(e) => {
                                error!("Failed to accept connection: {e}");
                                continue;
                            }
                        }

                    }
                    _ = shutdown_signal.next() => {
                        info!("Shutdown signal received, stopping listener loop");
                        break;
                    }
                }
            }
        });

        spawn_with_panic_notify_default(Self::event_handler_task(
            com_interface_proxy.event_receiver,
            tx_by_socket.clone(),
        ));

        Ok(InterfaceProperties {
            name: Some(format!("{}:{}", host, self.port)),
            ..Self::get_default_properties()
        })
    }

    #[allow(clippy::await_holding_lock)]
    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        mut receiver: UnboundedReceiver<ComInterfaceEvent>,
        tx_by_socket: Arc<
            Mutex<HashMap<ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>>>,
        >,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceEvent::SendBlock(block, socket_uuid) => {
                    let tx = tx_by_socket
                        .lock()
                        .map(|guard| guard.get(&socket_uuid).cloned());
                    let Ok(tx) = tx else {
                        error!("Client is not connected: {}", socket_uuid);
                        continue;
                    };
                    let mut tx = tx.unwrap();
                    if tx.start_send(block.to_bytes()).is_err() {
                        error!("Write failed for {}", socket_uuid);
                    }
                }
                ComInterfaceEvent::Destroy => {
                    break;
                }
                _ => todo!(),
            }
        }
    }

    async fn handle_receive(
        mut rx: OwnedReadHalf,
        mut bytes_in_sender: UnboundedSender<Vec<u8>>,
        mut shutdown_signal: BroadcastReceiver<()>,
    ) {
        let mut buffer = [0u8; 1024];
        loop {
            select! {
                result = rx.read(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            warn!("Connection closed by peer");
                            break;
                        }
                        Ok(n) => {
                            bytes_in_sender.start_send(buffer[..n].to_vec()).expect("Failed to send received data to ComHub");
                        }
                        Err(e) => {
                            error!("Failed to read from socket: {e}");
                            break;
                        }
                    }
                }

                // Shutdown signal received
                _ = shutdown_signal.next() => {
                    break;
                }
            }
        }
    }

    async fn handle_send(
        mut tcp_write_half: OwnedWriteHalf,
        mut tx_receiver: UnboundedReceiver<Vec<u8>>,
        mut shutdown_signal: BroadcastReceiver<()>,
    ) {
        loop {
            select! {
                maybe_block = tx_receiver.next() => {
                    match maybe_block {
                        Some(block) => {
                            if tcp_write_half.write_all(&block).await.is_err() {
                                error!("Failed to write to socket");
                                break;
                            }
                        }
                        None => {
                            // FIXME handle closed channel properly
                            continue;
                        }
                    }
                }
                _ = shutdown_signal.next() => {
                    break;
                }
            }
        }
    }
}

impl ComInterfaceAsyncFactory for TCPServerInterfaceSetupData {
    fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> ComInterfaceAsyncFactoryResult {
        Box::pin(
            async move { self.create_interface(com_interface_proxy).await },
        )
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

#[cfg(test)]
mod tests {
    use datex_macros::async_test;
    use std::assert_matches::assert_matches;

    use crate::{
        network::{
            com_hub::errors::InterfaceCreateError,
            com_interfaces::{
                com_interface::ComInterfaceProxy,
                default_com_interfaces::tcp::tcp_common::TCPServerInterfaceSetupData,
            },
        },
        runtime::AsyncContext,
    };

    #[async_test]
    async fn test_construct() {
        const PORT: u16 = 5088;
        let interface_properties =
            TCPServerInterfaceSetupData::create_interface(
                TCPServerInterfaceSetupData::new_with_port(PORT),
                ComInterfaceProxy::new_with_channels(AsyncContext::default()).0,
            )
            .await
            .unwrap();

        assert_eq!(
            interface_properties.name,
            Some(format!("0.0.0.0:{}", PORT))
        );
    }

    #[async_test]
    async fn test_invalid_address() {
        assert_matches!(
            TCPServerInterfaceSetupData::create_interface(
                TCPServerInterfaceSetupData::new_with_host_and_port(
                    "invalid-address".to_string(),
                    5088
                ),
                ComInterfaceProxy::new_with_channels(AsyncContext::default()).0
            )
            .await,
            Err(InterfaceCreateError::InvalidSetupData(_))
        );
    }
}
