use crate::std_sync::Mutex;
use crate::stdlib::cell::RefCell;
use crate::stdlib::rc::Rc;
use crate::stdlib::{
    collections::HashMap, future::Future, net::SocketAddr, pin::Pin,
};
use crate::{stdlib::sync::Arc, task::spawn};
use core::prelude::rust_2024::*;
use core::result::Result;
use core::time::Duration;
use datex_macros::{com_interface, create_opener};

use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    sync::Notify,
    task::JoinHandle,
};
use tungstenite::Message;
use url::Url;

use futures_util::stream::SplitSink;
use tokio_tungstenite::accept_async;

use super::websocket_common::{
    WebSocketError, WebSocketServerError, WebSocketServerInterfaceSetupData,
    parse_url,
};
use crate::network::com_interfaces::com_interface::ComInterface;
use crate::network::com_interfaces::com_interface::error::ComInterfaceError;
use crate::network::com_interfaces::com_interface::implementation::{ComInterfaceAsyncFactory, ComInterfaceSyncFactory};
use crate::network::com_interfaces::com_interface::implementation::ComInterfaceImplementation;
use crate::network::com_interfaces::com_interface::properties::{
    InterfaceDirection, InterfaceProperties,
};
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;
use crate::runtime::global_context::{get_global_context, set_global_context};
use tokio_tungstenite::WebSocketStream;
use crate::network::com_hub::errors::InterfaceCreateError;

type WebsocketStreamMap = HashMap<
    ComInterfaceSocketUUID,
    SplitSink<WebSocketStream<TcpStream>, Message>,
>;

pub struct WebSocketServerNativeInterface {
    pub websocket_streams_by_socket: Arc<Mutex<WebsocketStreamMap>>,
    shutdown_signal: Arc<Notify>,
    com_interface: Rc<ComInterface>,
}

impl WebSocketServerNativeInterface {

    async fn create(
        setup_data: WebSocketServerInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {

        let address: String = format!("0.0.0.0:{}", setup_data.port);
        let address = parse_url(&address, setup_data.secure.unwrap_or(true)).map_err(|_| {
            InterfaceCreateError::InvalidSetupData
        })?;

        info!("Spinning up server at {address}");
        let addr = format!(
            "{}:{}",
            address.host_str().unwrap(),
            address.port_or_known_default().unwrap()
        )
        .parse::<SocketAddr>()
        .map_err(|_| InterfaceCreateError::InvalidSetupData)?;

        let listener = TcpListener::bind(&addr).await.map_err(|err| ComInterfaceError::connection_error_with_details(err))?;

        let websocket_streams = Arc::new(Mutex::new(HashMap::new()));
        let websocket_streams_clone = websocket_streams.clone();
        let shutdown_signal = Arc::new(Notify::new());
        let mut tasks: Vec<JoinHandle<()>> = vec![];
        let global_context = get_global_context();

        let manager = com_interface.socket_manager();

        let shutdown_signal_clone = shutdown_signal.clone();

        spawn(async move {
            let global_context = global_context.clone();
            let manager = manager.clone();
            set_global_context(global_context.clone());
            info!("WebSocket server started at {addr}");
            loop {
                let manager = manager.clone();
                select! {
                    res = listener.accept() => {
                        match res {
                            Ok((stream, addr)) => {
                                let websocket_streams = websocket_streams_clone.clone();
                                let global_context = global_context.clone();
                                info!("New connection from {addr}");
                                let task = spawn(async move {
                                    set_global_context(global_context.clone());
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
                                tasks.push(task);
                            }
                            Err(e) => {
                                error!("Failed to accept connection: {e}");
                                continue;
                            }
                        };
                    }
                    _ = shutdown_signal_clone.notified() => {
                        info!("Shutdown signal received, stopping server...");
                        for task in tasks {
                            task.abort();
                        }
                        break;
                    }
                }
            }
        });

        Ok((
            WebSocketServerNativeInterface {
                websocket_streams_by_socket: websocket_streams,
                shutdown_signal,
                com_interface,
            },
            InterfaceProperties {
                name: Some(address.to_string()),
                ..Self::get_default_properties()
            }
        ))
    }
}

impl ComInterfaceAsyncFactory for WebSocketServerNativeInterface {
    type SetupData = WebSocketServerInterfaceSetupData;

    fn create(
        setup_data: Self::SetupData,
        com_interface: Rc<ComInterface>,
    ) -> Pin<Box<dyn Future<Output = Result<(Self, InterfaceProperties), InterfaceCreateError>> + 'static>> {
        Box::pin(async move {
            WebSocketServerNativeInterface::create(setup_data, com_interface).await
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

impl ComInterfaceImplementation for WebSocketServerNativeInterface {
    fn send_block<'a>(
        &'a self,
        block: &'a [u8],
        socket_uuid: ComInterfaceSocketUUID,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        let tx = self.websocket_streams_by_socket.clone();
        Box::pin(async move {
            let tx = &mut tx.try_lock().unwrap();
            let tx = tx.get_mut(&socket_uuid);
            if tx.is_none() {
                error!("Client is not connected");
                return false;
            }
            tx.unwrap()
                .send(Message::Binary(block.to_vec()))
                .await
                .map_err(|e| {
                    error!("Error sending message: {e:?}");
                    false
                })
                .is_ok()
        })
    }

    fn handle_destroy<'a>(&'a self) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        let shutdown_signal = self.shutdown_signal.clone();
        let websocket_streams = self.websocket_streams_by_socket.clone();
        Box::pin(async move {
            shutdown_signal.notify_waiters();
            websocket_streams.try_lock().unwrap().clear();
            true
        })
    }

    fn handle_reconnect<'a>(&'a self) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        todo!()
    }
}
