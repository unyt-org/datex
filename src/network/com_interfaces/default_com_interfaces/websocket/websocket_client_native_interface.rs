use crate::{
    stdlib::{
        sync::{Arc},
        time::Duration,
    },
    task::spawn_with_panic_notify,
};
use core::{prelude::rust_2024::*, result::Result};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use log::{error, info, warn};
use tokio::net::TcpStream;
use tungstenite::Message;
use url::Url;
use futures::lock::Mutex;

use super::websocket_common::{WebSocketClientInterfaceSetupData, parse_url};
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
            state::{ComInterfaceState, ComInterfaceStateWrapper},
        },
    },
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use crate::global::dxb_block::DXBBlock;
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SocketDataIterator, NewSocketsIterator, SendCallback, SendFailure, SocketConfiguration};
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;

impl WebSocketClientInterfaceSetupData {
    async fn create_interface(
        self,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let (address, write, mut read) =
            self.create_websocket_client_connection().await?;
        let write = Arc::new(Mutex::new(write));

        Ok(
            ComInterfaceConfiguration {
                properties: InterfaceProperties {
                    name: Some(address.to_string()),
                    ..Self::get_default_properties()
                },
                send_callback: SendCallback::new_async(move |(block, _uuid): (DXBBlock, ComInterfaceSocketUUID)| {
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
                }),
                close_callback: None,
                new_sockets_iterator: NewSocketsIterator::new_single(SocketDataIterator::new(
                    SocketConfiguration::new(InterfaceDirection::InOut, 1),
                    async gen move {
                        loop {
                            match read.next().await {
                                Some(Ok(Message::Binary(data))) => {
                                    yield Ok(data);
                                }
                                Some(Ok(_)) => {
                                    error!("Invalid message type received");
                                    return yield Err(());
                                }
                                Some(Err(e)) => {
                                    error!("WebSocket read error: {e}");
                                    return yield Err(());
                                }
                                None => {
                                    warn!("WebSocket closed by peer");
                                    return yield Err(())
                                }
                            }
                        }
                    }
                )),
            }
        )
    }

    /// initialize a new websocket client connection
    async fn create_websocket_client_connection(
        &self,
    ) -> Result<
        (
            Url,
            SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
            SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        ),
        ComInterfaceCreateError,
    > {
        let address = parse_url(&self.url).map_err(|_| {
            ComInterfaceCreateError::InvalidSetupData(
                "Invalid WebSocket URL".to_string(),
            )
        })?;
        if address.scheme() != "ws" && address.scheme() != "wss" {
            return Err(ComInterfaceCreateError::InvalidSetupData(
                "Invalid WebSocket URL scheme".to_string(),
            ));
        }
        info!("Connecting to WebSocket server at {address}");
        let (stream, _) = tokio_tungstenite::connect_async(address.clone())
            .await
            .map_err(|e| {
                error!("Failed to connect to WebSocket server: {e}");
                ComInterfaceCreateError::InterfaceError(
                    ComInterfaceError::connection_error_with_details(
                        e.to_string(),
                    ),
                )
            })?;
        let (write, read) = stream.split();
        Ok((address, write, read))
    }
}

impl ComInterfaceAsyncFactory for WebSocketClientInterfaceSetupData {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
    }

    fn get_default_properties() -> InterfaceProperties {
        InterfaceProperties {
            interface_type: "websocket-client".to_string(),
            channel: "websocket".to_string(),
            round_trip_time: Duration::from_millis(40),
            max_bandwidth: 1000,
            ..InterfaceProperties::default()
        }
    }
}
