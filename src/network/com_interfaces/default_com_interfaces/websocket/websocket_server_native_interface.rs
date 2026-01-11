use crate::std_sync::Mutex;
use crate::stdlib::rc::Rc;
use crate::stdlib::sync::Arc;
use crate::stdlib::{
    collections::HashMap, future::Future, net::SocketAddr, pin::Pin,
};
use crate::task::{spawn_with_panic_notify_default, UnboundedReceiver};
use core::prelude::rust_2024::*;
use core::result::Result;
use core::time::Duration;

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
use tokio_tungstenite::{accept_async, MaybeTlsStream};

use super::websocket_common::{WebSocketServerInterfaceSetupData, parse_url};
use crate::network::com_hub::errors::InterfaceCreateError;
use crate::network::com_interfaces::com_interface::{ComInterface, ComInterfaceImplEvent};
use crate::network::com_interfaces::com_interface::error::ComInterfaceError;
use crate::network::com_interfaces::com_interface::implementation::ComInterfaceImplementation;
use crate::network::com_interfaces::com_interface::implementation::{
    ComInterfaceAsyncFactory, ComInterfaceSyncFactory,
};
use crate::network::com_interfaces::com_interface::properties::{
    InterfaceDirection, InterfaceProperties,
};
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;
use crate::runtime::global_context::get_global_context;
use tokio_tungstenite::WebSocketStream;
use crate::network::com_interfaces::com_interface::state::{ComInterfaceState, ComInterfaceStateWrapper};

type WebsocketStreamMap = HashMap<
    ComInterfaceSocketUUID,
    SplitSink<WebSocketStream<TcpStream>, Message>,
>;

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
        let shutdown_signal = Arc::new(Notify::new());
        let tasks: Vec<JoinHandle<()>> = vec![];
        let global_context = get_global_context();

        let manager = com_interface.socket_manager();

        let shutdown_signal_clone = shutdown_signal.clone();

        // FIXME fix task abort on shutdown
        spawn_with_panic_notify_default(async move {
            let manager = manager.clone();
            info!("WebSocket server started at {addr}");
            loop {
                let manager = manager.clone();
                select! {
                    res = listener.accept() => {
                        match res {
                            Ok((stream, addr)) => {
                                let websocket_streams = websocket_streams_clone.clone();
                                info!("New connection from {addr}");
                                spawn_with_panic_notify_default(async move {
                                    let manager = manager.clone();

                                    match accept_async(stream).await {
                                        Ok(ws_stream) => {
                                            info!(
                                                "Accepted WebSocket connection from {addr}"
                                            );
                                            let (write, mut read) = ws_stream.split();

                                            let (socket_uuid, mut sender) = manager
                                                .lock()
                                                .unwrap()
                                                .create_and_init_socket(InterfaceDirection::InOut, 1);

                                            websocket_streams
                                                .try_lock()
                                                .unwrap()
                                                .insert(socket_uuid.clone(), write);

                                            while let Some(msg) = read.next().await {
                                                match msg {
                                                    Ok(Message::Binary(bin)) => {
                                                        sender.start_send(bin).unwrap();
                                                    }
                                                    Ok(_) => {}
                                                    Err(e) => {
                                                        error!(
                                                            "WebSocket error from {addr}: {e}"
                                                        );
                                                        break;
                                                    }
                                                }
                                            }
                                            // consider the connection closed, clean up
                                            let mut streams =
                                                websocket_streams
                                                    .try_lock()
                                                    .unwrap();
                                            streams.remove(&socket_uuid);

                                            manager
                                                .lock()
                                                .unwrap()
                                                .remove_socket(socket_uuid);
                                            info!(
                                                "WebSocket connection from {addr} closed"
                                            );
                                        }
                                        Err(e) => {
                                            error!(
                                                "WebSocket handshake failed with {addr}: {e}"
                                            );
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                error!("Failed to accept connection: {e}");
                                continue;
                            }
                        };
                    }
                    _ = shutdown_signal_clone.notified() => {
                        info!("Shutdown signal received, stopping server...");
                        // for task in tasks {
                        //     task.abort();
                        // }
                        break;
                    }
                }
            }
        });

        // start event handler task
        let interface_impl_event_receiver = com_interface
            .take_interface_impl_event_receiver();
        let shutdown_signal_clone = shutdown_signal.clone();
        let websocket_streams_clone = websocket_streams_by_socket.clone();
        spawn_with_panic_notify_default(async move {
            Self::event_handler_task(
                websocket_streams_clone,
                interface_impl_event_receiver,
                shutdown_signal_clone,
            )
            .await;
        });

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

    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        mut websocket_streams_by_socket: Arc<Mutex<WebsocketStreamMap>>,
        mut receiver: UnboundedReceiver<ComInterfaceImplEvent>,
        shutdown_signal: Arc<Notify>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceImplEvent::SendBlock(block, socket_uuid) => {
                    let tx = &mut websocket_streams_by_socket.try_lock().unwrap();
                    let tx = tx.get_mut(&socket_uuid);
                    match tx {
                        Some(tx) => {
                            tx
                                .send(Message::Binary(block.to_vec()))
                                .await
                                .unwrap();
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
                    shutdown_signal.notify_waiters();
                }
                _ => todo!()
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
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        (Self, InterfaceProperties),
                        InterfaceCreateError,
                    >,
                > + 'static,
        >,
    > {
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