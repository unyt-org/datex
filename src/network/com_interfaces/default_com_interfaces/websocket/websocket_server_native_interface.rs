use crate::{
    network::com_interfaces::com_interface::{
        socket_manager::ComInterfaceSocketManager,
        state::ComInterfaceStateWrapper,
    },
    std_sync::Mutex,
    stdlib::{collections::HashMap, net::SocketAddr, sync::Arc},
    task::{
        UnboundedReceiver, UnboundedSender, create_unbounded_channel,
        spawn_with_panic_notify_default,
    },
};
use core::{
    prelude::rust_2024::*, result::Result, str::FromStr, time::Duration,
};
use futures::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tungstenite::Message;

use async_select::select;
use futures_util::stream::SplitSink;
use tokio_tungstenite::accept_async;

use super::websocket_common::{
    WebSocketClientInterfaceSetupData, WebSocketServerInterfaceSetupData,
    parse_url,
};
use crate::{
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterfaceEvent,
            error::ComInterfaceError,
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
                ComInterfaceSyncFactory,
            },
            properties::{InterfaceDirection, InterfaceProperties},
            socket::ComInterfaceSocketUUID,
        },
    },
    runtime::RuntimeConfigInterface,
};
use datex_core::network::com_interfaces::com_interface::ComInterfaceProxy;
use tokio_tungstenite::WebSocketStream;

type WebsocketStreamMap =
    HashMap<ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>>;

impl WebSocketServerInterfaceSetupData {
    async fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> Result<InterfaceProperties, InterfaceCreateError> {
        let addr = SocketAddr::from_str(&self.bind_address)
            .map_err(InterfaceCreateError::invalid_setup_data)?;

        info!("Spinning up server at {addr}");

        let listener = TcpListener::bind(&addr).await.map_err(|err| {
            ComInterfaceError::connection_error_with_details(err)
        })?;

        let websocket_streams_by_socket = Arc::new(Mutex::new(HashMap::new()));
        let websocket_streams_clone = websocket_streams_by_socket.clone();
        let shutdown_signal = com_interface_proxy.shutdown_signal();
        let manager = com_interface_proxy.socket_manager;
        let state = com_interface_proxy.state;

        spawn_with_panic_notify_default(async move {
            let manager = manager.clone();
            info!("WebSocket server started at {addr}");
            loop {
                let manager = manager.clone();
                select! {
                    res = listener.accept() => {
                        match res {
                            Ok((stream, addr)) => {
                                let state = state.clone();
                                let websocket_streams = websocket_streams_clone.clone();
                                info!("New connection from {addr}");
                                match accept_async(stream).await {
                                    Ok(ws_stream) => {
                                        let (write, read) = ws_stream.split();
                                        let (tx_sender, tx_receiver) = create_unbounded_channel::<Vec<u8>>();

                                        info!("Accepted WebSocket connection from {addr}");
                                        let manager = manager.clone();

                                        let (socket_uuid, sender) = manager
                                            .lock()
                                            .unwrap()
                                            .create_and_init_socket_with_optional_endpoint(InterfaceDirection::InOut, 1, None);

                                        websocket_streams
                                            .try_lock()
                                            .unwrap()
                                            .insert(socket_uuid.clone(), tx_sender);

                                        let state_clone = state.clone();

                                        // spawn read task
                                        spawn_with_panic_notify_default(async move {
                                            Self::client_read_task(
                                                manager,
                                                read,
                                                sender,
                                                addr,
                                                websocket_streams,
                                                state_clone,
                                                socket_uuid
                                            )
                                            .await;
                                        });

                                        // spawn write task
                                        spawn_with_panic_notify_default(async move {
                                            Self::client_write_task(
                                                write,
                                                tx_receiver,
                                                addr,
                                                state,
                                            )
                                            .await;
                                        });
                                    }
                                    Err(e) => {
                                        error!("WebSocket handshake failed with {addr}: {e}");
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to accept connection: {e}");
                                continue;
                            }
                        };
                    }
                    _ = shutdown_signal.notified() => {
                        break;
                    }
                }
            }
        });

        // start event handler task
        let websocket_streams_clone = websocket_streams_by_socket.clone();
        spawn_with_panic_notify_default(Self::event_handler_task(
            websocket_streams_clone,
            com_interface_proxy.event_receiver,
        ));

        Ok(InterfaceProperties {
            name: Some(addr.to_string()),
            connectable_interfaces: self.accept_addresses.map(|addrs| {
                addrs
                    .into_iter()
                    .map(|(address, tls_mode)| {
                        let url = format!(
                            "{}://{}",
                            if tls_mode.is_some() { "wss" } else { "ws" },
                            address
                        );
                        // parse and validate URL
                        parse_url(&url).map_err(|_| {
                            InterfaceCreateError::invalid_setup_data(
                                format!("Invalid URL for WebSocket connection: {url}")
                            )
                        })?;
                        RuntimeConfigInterface::new(
                            "websocket-client",
                            WebSocketClientInterfaceSetupData {
                                url,
                            },
                        ).map_err(|e| {
                            InterfaceCreateError::invalid_setup_data(
                                format!("Failed to create connectable interface for WebSocket client: {e}")
                            )
                        })
                    })
                    .collect::<_>()
            })
                .transpose()?,
            ..Self::get_default_properties()
        })
    }

    async fn client_write_task(
        mut write: SplitSink<WebSocketStream<TcpStream>, Message>,
        mut receiver: UnboundedReceiver<Vec<u8>>,
        addr: SocketAddr,
        state: Arc<Mutex<ComInterfaceStateWrapper>>,
    ) {
        let shutdown_signal = state.try_lock().unwrap().shutdown_signal();
        loop {
            select! {
                // Receive next message to send
                msg = receiver.next() => {
                    match msg {
                        Some(data) => {
                            if let Err(e) = write.send(Message::Binary(data)).await {
                                error!("WebSocket write error to {addr}: {e}");
                                continue;
                            }
                        }
                        None => {
                            // Channel closed
                            break;
                        }
                    }
                }
                // Shutdown signal received
                _ = shutdown_signal.notified() => {
                    info!("Shutdown signal received, stopping write_task for {addr}");
                    break;
                }
            }
        }
    }

    async fn client_read_task(
        manager: Arc<Mutex<ComInterfaceSocketManager>>,
        mut read: SplitStream<WebSocketStream<TcpStream>>,
        mut sender: UnboundedSender<Vec<u8>>,
        addr: SocketAddr,
        websocket_streams: Arc<Mutex<WebsocketStreamMap>>,
        state: Arc<Mutex<ComInterfaceStateWrapper>>,
        socket_uuid: ComInterfaceSocketUUID,
    ) {
        let shutdown_signal = state.try_lock().unwrap().shutdown_signal();

        loop {
            select! {
                // Read next WebSocket message
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Binary(bin))) => {
                            sender.start_send(bin).expect("Failed to send received data to ComHub");
                        }
                        Some(Ok(_)) => {
                            // Ignore non-binary messages
                            continue;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error from {addr}: {e}");
                            continue;
                        }
                        None => {
                            // Connection closed by peer
                            break;
                        }
                    }
                }
                // Shutdown signal received
                _ = shutdown_signal.notified() => {
                    info!("Shutdown signal received, stopping read_task for {addr}");
                    break;
                }
            }
        }

        // cleanup on connection close
        let mut streams = websocket_streams.try_lock().unwrap();
        streams.remove(&socket_uuid);
        manager.lock().unwrap().remove_socket(socket_uuid);
        info!("WebSocket connection from {addr} closed");
    }

    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        websocket_streams_by_socket: Arc<Mutex<WebsocketStreamMap>>,
        mut receiver: UnboundedReceiver<ComInterfaceEvent>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceEvent::SendBlock(block, socket_uuid) => {
                    let tx =
                        &mut websocket_streams_by_socket.try_lock().unwrap();
                    let tx = tx.get_mut(&socket_uuid);
                    match tx {
                        Some(tx) => {
                            tx.start_send(block.to_bytes()).expect(
                                "Failed to send outgoing data to WebSocket",
                            );
                        }
                        None => {
                            error!(
                                "Socket UUID {:?} not found for sending",
                                socket_uuid
                            );
                        }
                    };
                }
                ComInterfaceEvent::Destroy => {
                    break;
                }
                _ => todo!(),
            }
        }
    }
}

impl ComInterfaceAsyncFactory for WebSocketServerInterfaceSetupData {
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
            interface_type: "websocket-server".to_string(),
            channel: "websocket".to_string(),
            round_trip_time: Duration::from_millis(40),
            max_bandwidth: 1000,
            ..InterfaceProperties::default()
        }
    }
}
