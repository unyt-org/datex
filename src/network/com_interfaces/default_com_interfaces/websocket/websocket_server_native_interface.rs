use crate::{
    stdlib::{collections::HashMap, net::SocketAddr, sync::Arc},
};
use core::{
    prelude::rust_2024::*, result::Result, str::FromStr, time::Duration,
};
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures::lock::Mutex;

use super::websocket_common::{WebSocketClientInterfaceSetupData, WebSocketServerInterfaceSetupData, parse_url, TLSMode};
use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            error::ComInterfaceError,
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::{InterfaceDirection, ComInterfaceProperties},
            socket::ComInterfaceSocketUUID,
        },
    },
    runtime::RuntimeConfigInterface,
};
use crate::global::dxb_block::DXBBlock;
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SendFailure, SendSuccess, SocketProperties, SocketConfiguration};

type WebsocketStreamMap =
    HashMap<ComInterfaceSocketUUID, Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>;

impl WebSocketServerInterfaceSetupData {
    async fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let addr = SocketAddr::from_str(&self.bind_address)
            .map_err(ComInterfaceCreateError::invalid_setup_data)?;

        info!("Spinning up server at {addr}");

        let listener = TcpListener::bind(&addr).await.map_err(|err| {
            ComInterfaceCreateError::connection_error_with_details(err)
        })?;

        Ok(ComInterfaceConfiguration::new(
            ComInterfaceProperties {
                name: Some(addr.to_string()),
                connectable_interfaces: Self::get_connectable_interface_configs_from_accept_addresses(
                    self.accept_addresses
                )?,
                ..Self::get_default_properties()
            },
            async gen move {
                loop {
                    // get next websocket connection
                    match Self::get_next_websocket_connection(&listener).await {
                        Ok((mut read, write)) => {
                            info!("Accepted new WebSocket connection");
                            // yield new socket data
                            yield Ok(SocketConfiguration::new(
                                SocketProperties::new(InterfaceDirection::InOut, 1),
                                // socket incoming blocks iterator
                                async gen move {
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
                                                return;
                                            }
                                        }
                                    }
                                },
                                // socket send callback
                                SendCallback::new_async(move |block: DXBBlock| {
                                    let write = write.clone();
                                    async move {
                                        write
                                            .lock()
                                            .await
                                            .send(Message::Binary(block.to_bytes())).await
                                            .map_err(|e| {
                                                error!("WebSocket write error: {e}");
                                                SendFailure(block)
                                            })
                                    }
                                })
                            ));
                        }
                        Err(_) => {
                            // Failed to accept connection, continue to next
                            continue;
                        }
                    }
                }
            }
        ))
    }

    async fn get_next_websocket_connection(listener: &TcpListener) -> Result<
        (SplitStream<WebSocketStream<TcpStream>>, Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>),
        ()
    > {
        // new sockets iterators are yielded on client connection
        let next_socket = listener.accept().await;
        match next_socket {
            Ok((stream, addr)) => {
                info!("New connection from {addr}");
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        let (write, mut read) = ws_stream.split();
                        let write = Arc::new(Mutex::new(write));
                        Ok((read, write))
                    }
                    Err(e) => {
                        error!("WebSocket handshake failed with {addr}: {e}");
                        Err(())
                    }
                }
            }
            Err(e) => {
                error!("Failed to accept connection: {e}");
                Err(())
            }
        }
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

    fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "websocket-server".to_string(),
            channel: "websocket".to_string(),
            round_trip_time: Duration::from_millis(40),
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
}
