use crate::{
    channel::mpsc::{
        UnboundedReceiver, UnboundedSender, create_unbounded_channel,
    },
    network::com_interfaces::com_interface::{
        socket_manager::ComInterfaceSocketManager,
        state::ComInterfaceStateWrapper,
    },
    std_sync::Mutex,
    stdlib::{collections::HashMap, net::SocketAddr, sync::Arc},
};
use core::{
    prelude::rust_2024::*, result::Result, str::FromStr, time::Duration,
};
use futures::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tungstenite::Message;
use tokio_tungstenite::accept_async;

use super::websocket_common::{WebSocketClientInterfaceSetupData, WebSocketServerInterfaceSetupData, parse_url, TLSMode};
use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterfaceEvent,
            error::ComInterfaceError,
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::{InterfaceDirection, InterfaceProperties},
            socket::ComInterfaceSocketUUID,
        },
    },
    runtime::RuntimeConfigInterface,
};
use crate::global::dxb_block::DXBBlock;
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, NewSocketsIterator, SendCallback, SendFailure, SendSuccess, SocketConfiguration, SocketDataIterator};

type WebsocketStreamMap =
    HashMap<ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>>;

impl WebSocketServerInterfaceSetupData {
    async fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let addr = SocketAddr::from_str(&self.bind_address)
            .map_err(ComInterfaceCreateError::invalid_setup_data)?;

        info!("Spinning up server at {addr}");

        let listener = TcpListener::bind(&addr).await.map_err(|err| {
            ComInterfaceError::connection_error_with_details(err)
        })?;

        let websocket_streams_by_socket = Arc::new(Mutex::<WebsocketStreamMap>::new(HashMap::new()));
        let websocket_streams_clone = websocket_streams_by_socket.clone();

        Ok(ComInterfaceConfiguration {
            properties: InterfaceProperties {
                name: Some(addr.to_string()),
                connectable_interfaces: Self::get_connectable_interface_configs_from_accept_addresses(
                    self.accept_addresses
                )?,
                ..Self::get_default_properties()
            },
            send_callback: SendCallback::new_sync(move |(block, uuid): (DXBBlock, ComInterfaceSocketUUID)| {
                let tx =
                    &mut websocket_streams_by_socket.try_lock().unwrap();
                let tx = tx.get_mut(&uuid);
                match tx {
                    Some(tx) => {
                        tx.start_send(block.to_bytes()).expect(
                            "Failed to send outgoing data to WebSocket",
                        );
                        Ok(SendSuccess::Sent)
                    }
                    None => {
                        error!(
                                "Socket UUID {:?} not found for sending",
                                uuid
                            );
                        Err(SendFailure(block))
                    }
                }
            }),
            close_callback: None,
            new_sockets_iterator: NewSocketsIterator::new_multiple(async gen move {
                loop {
                    // new sockets iterators are yielded on client connection
                    let next_socket = listener.accept().await;
                    match next_socket {
                        Ok((stream, addr)) => {
                            let websocket_streams = websocket_streams_clone.clone();
                            info!("New connection from {addr}");

                            let socket_configuration =  SocketConfiguration::new(InterfaceDirection::InOut, 1);
                            let socket_uuid = socket_configuration.uuid();

                            // yield new socket data
                            yield Ok(SocketDataIterator::new(
                                socket_configuration,
                                async gen move {
                                    match accept_async(stream).await {
                                        Ok(ws_stream) => {
                                            let (write, mut read) = ws_stream.split();
                                            let (tx_sender, tx_receiver) = create_unbounded_channel::<Vec<u8>>();

                                            info!("Accepted WebSocket connection from {addr}");

                                            websocket_streams
                                                .try_lock()
                                                .unwrap()
                                                .insert(socket_uuid, tx_sender);

                                            // read blocks
                                            loop {
                                                match read.next().await {
                                                    Some(Ok(Message::Binary(bin))) => {
                                                        yield Ok(bin);
                                                    }
                                                    Some(Ok(_)) => {
                                                        error!("Invalid message type received");
                                                        return yield Err(());
                                                    }
                                                    Some(Err(e)) => {
                                                        error!("WebSocket error from {addr}: {e}");
                                                        return yield Err(())
                                                    }
                                                    None => {
                                                        // Connection closed by peer
                                                        return yield Err(())
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("WebSocket handshake failed with {addr}: {e}");
                                            // immediately close the socket again
                                            yield Err(());
                                        }
                                    }
                                }
                            ));
                        },
                        Err(e) => {
                            error!("Failed to accept connection: {e}");
                            continue;
                        }
                    }
                }
            }),
        })
    }

    fn get_connectable_interface_configs_from_accept_addresses(
        accept_addresses: Option<Vec<(String, Option<TLSMode>)>>,
    ) -> Result<Option<Vec<RuntimeConfigInterface>>, ComInterfaceCreateError> {
        accept_addresses.map(|addrs| {
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
                        ComInterfaceCreateError::invalid_setup_data(
                            format!("Invalid URL for WebSocket connection: {url}")
                        )
                    })?;
                    RuntimeConfigInterface::new(
                        "websocket-client",
                        WebSocketClientInterfaceSetupData {
                            url,
                        },
                    ).map_err(|e| {
                        ComInterfaceCreateError::invalid_setup_data(
                            format!("Failed to create connectable interface for WebSocket client: {e}")
                        )
                    })
                })
                .collect::<_>()
        }).transpose()
    }
}

impl ComInterfaceAsyncFactory for WebSocketServerInterfaceSetupData {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
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
