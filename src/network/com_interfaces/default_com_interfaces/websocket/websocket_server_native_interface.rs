use crate::{
    network::com_interfaces::com_interface::{
        socket_manager::ComInterfaceSocketManager,
        state::ComInterfaceStateWrapper,
    },
    std_sync::Mutex,
    stdlib::{collections::HashMap, net::SocketAddr, rc::Rc, sync::Arc},
    task::{
        UnboundedReceiver, UnboundedSender, create_unbounded_channel,
        spawn_with_panic_notify_default,
    },
};
use core::{prelude::rust_2024::*, result::Result, time::Duration};

use futures::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    sync::Notify,
    task::JoinHandle,
};
use tungstenite::Message;

use futures_util::stream::SplitSink;
use tokio_tungstenite::accept_async;

use super::websocket_common::{WebSocketServerInterfaceSetupData, parse_url};
use crate::{
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterface, ComInterfaceImplEvent,
            error::ComInterfaceError,
            implementation::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
                ComInterfaceImplementation, ComInterfaceSyncFactory,
            },
            properties::{InterfaceDirection, InterfaceProperties},
            socket::ComInterfaceSocketUUID,
        },
    },
    runtime::global_context::get_global_context,
};
use tokio_tungstenite::WebSocketStream;

type WebsocketStreamMap =
    HashMap<ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>>;

pub struct WebSocketServerNativeInterface {
    // TODO: properties not really needed here, just for testing purposes
    pub websocket_streams_by_socket: Arc<Mutex<WebsocketStreamMap>>,
    com_interface: Rc<ComInterface>,
}

impl WebSocketServerNativeInterface {
    async fn create(
        setup_data: WebSocketServerInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
        let address: String = format!(
            "{}://0.0.0.0:{}",
            match setup_data.secure.unwrap_or(true) {
                true => "wss",
                false => "ws",
            },
            setup_data.port
        );
        let address = parse_url(&address)
            .map_err(InterfaceCreateError::invalid_setup_data)?;

        info!("Spinning up server at {address}");
        let addr = format!(
            "{}:{}",
            address.host_str().unwrap(),
            address.port_or_known_default().unwrap()
        )
        .parse::<SocketAddr>()
        .map_err(InterfaceCreateError::invalid_setup_data)?;

        let listener = TcpListener::bind(&addr).await.map_err(|err| {
            ComInterfaceError::connection_error_with_details(err)
        })?;

        let websocket_streams_by_socket = Arc::new(Mutex::new(HashMap::new()));
        let websocket_streams_clone = websocket_streams_by_socket.clone();
        let shutdown_signal = com_interface.shutdown_signal();
        let manager = com_interface.socket_manager();
        let state = com_interface.state();

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
                                            .create_and_init_socket(InterfaceDirection::InOut, 1);

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
        let interface_impl_event_receiver =
            com_interface.take_interface_impl_event_receiver();
        let websocket_streams_clone = websocket_streams_by_socket.clone();
        spawn_with_panic_notify_default(Self::event_handler_task(
            websocket_streams_clone,
            interface_impl_event_receiver,
        ));

        Ok((
            WebSocketServerNativeInterface {
                websocket_streams_by_socket,
                com_interface,
            },
            InterfaceProperties {
                name: Some(address.to_string()),
                ..Self::get_default_properties()
            },
        ))
    }

    async fn client_write_task(
        mut write: SplitSink<WebSocketStream<TcpStream>, Message>,
        mut receiver: UnboundedReceiver<Vec<u8>>,
        addr: SocketAddr,
        state: Arc<Mutex<ComInterfaceStateWrapper>>,
    ) {
        let shutdown_signal = state.try_lock().unwrap().shutdown_signal();
        loop {
            tokio::select! {
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
            tokio::select! {
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
        mut receiver: UnboundedReceiver<ComInterfaceImplEvent>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceImplEvent::SendBlock(block, socket_uuid) => {
                    let tx =
                        &mut websocket_streams_by_socket.try_lock().unwrap();
                    let tx = tx.get_mut(&socket_uuid);
                    match tx {
                        Some(tx) => {
                            tx.start_send(block.to_vec()).expect(
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
                ComInterfaceImplEvent::Destroy => {
                    break;
                }
                _ => todo!(),
            }
        }
    }
}

impl ComInterfaceImplementation for WebSocketServerNativeInterface {}

impl ComInterfaceAsyncFactory for WebSocketServerNativeInterface {
    type SetupData = WebSocketServerInterfaceSetupData;

    fn create(
        setup_data: Self::SetupData,
        com_interface: Rc<ComInterface>,
    ) -> ComInterfaceAsyncFactoryResult<Self> {
        Box::pin(async move {
            WebSocketServerNativeInterface::create(setup_data, com_interface)
                .await
        })
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
