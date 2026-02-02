use datex::{derive_setup_data};
use core::{ result::Result};
use std::sync::Arc;
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use log::{error, info, warn};
use tokio::net::TcpStream;
use tungstenite::Message;
use url::Url;
use futures::lock::Mutex;

use datex::network::com_interfaces::default_setup_data::websocket::websocket_client::{WebSocketClientInterfaceSetupData};
use datex::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::{InterfaceDirection, ComInterfaceProperties},
        },
    },
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use datex::network::com_interfaces::default_setup_data::http_common::parse_url;
use datex::global::dxb_block::DXBBlock;
use datex::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SocketConfiguration, SendCallback, SendFailure, SocketProperties};

derive_setup_data!(WebSocketClientInterfaceSetupDataNative, WebSocketClientInterfaceSetupData);

impl WebSocketClientInterfaceSetupDataNative {
    async fn create_interface(
        self,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let (_address, write, mut read) =
            self.create_websocket_client_connection().await?;
        let write = Arc::new(Mutex::new(write));

        Ok(
            ComInterfaceConfiguration::new_single_socket(
                ComInterfaceProperties {
                    name: Some(self.url.clone()),
                    ..Self::get_default_properties()
                },
                SocketConfiguration::new(
                    SocketProperties::new(InterfaceDirection::InOut, 1),
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
                                    return;
                                }
                            }
                        }
                    },
                    SendCallback::new_async(move |block: DXBBlock| {
                        let write = write.clone();
                        async move {
                            write
                                .lock()
                                .await
                                .send(Message::Binary(block.to_bytes())).await
                                .map_err(|e| {
                                    error!("WebSocket write error: {e}");
                                    SendFailure(Box::new(block))
                                })
                        }
                    })
                )
            )
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
                ComInterfaceCreateError::connection_error_with_details(
                    e.to_string(),
                )
            })?;
        let (write, read) = stream.split();
        Ok((address, write, read))
    }
}

impl ComInterfaceAsyncFactory for WebSocketClientInterfaceSetupDataNative {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
    }

    fn get_default_properties() -> ComInterfaceProperties {
        WebSocketClientInterfaceSetupData::get_default_properties()
    }
}
