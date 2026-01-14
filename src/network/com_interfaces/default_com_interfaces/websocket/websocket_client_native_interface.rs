use crate::stdlib::{
    sync::{Arc, Mutex},
    time::Duration,
};
use core::{prelude::rust_2024::*, result::Result};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use log::{error, info};
use tokio::net::TcpStream;
use tungstenite::Message;
use url::Url;

use super::websocket_common::{WebSocketClientInterfaceSetupData, parse_url};
use crate::{
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterfaceEvent,
            error::ComInterfaceError,
            implementation::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::{InterfaceDirection, InterfaceProperties},
            state::{ComInterfaceState, ComInterfaceStateWrapper},
        },
    },
    task::{
        UnboundedReceiver, UnboundedSender, spawn_with_panic_notify_default,
    },
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use datex_core::network::com_interfaces::com_interface::ComInterfaceProxy;
use crate::task::spawn_with_panic_notify;

impl WebSocketClientInterfaceSetupData {
    async fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> Result<InterfaceProperties, InterfaceCreateError> {
        let (address, write, read) =
            self.create_websocket_client_connection().await?;

        let (_, sender) = com_interface_proxy
            .create_and_init_socket(InterfaceDirection::InOut, 1);

        let state = com_interface_proxy.state;

        spawn_with_panic_notify(&com_interface_proxy.async_context, Self::read_task(
            read,
            sender,
            state.clone(),
        ));

        spawn_with_panic_notify(&com_interface_proxy.async_context, Self::event_handler_task(
            write,
            com_interface_proxy.event_receiver,
            state,
        ));

        Ok(InterfaceProperties {
            name: Some(address.to_string()),
            ..Self::get_default_properties()
        },)
    }

    /// background task to read messages from the websocket
    async fn read_task(
        mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        mut sender: UnboundedSender<Vec<u8>>,
        state: Arc<Mutex<ComInterfaceStateWrapper>>,
    ) {
        let shutdown_signal = state.try_lock().unwrap().shutdown_signal();
        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Binary(data))) => {
                            sender.start_send(data).expect("Failed to send received data to ComHub")
                        }
                        Some(Ok(_)) => {
                            error!("Invalid message type received");
                        }
                        Some(Err(e)) => {
                            error!("WebSocket read error: {e}");
                            // FIXME what about read errors that are not fatal?
                            continue;
                        }
                        None => {
                            log::warn!("WebSocket closed by peer");
                            state.lock().unwrap().set(ComInterfaceState::Destroyed);
                            break;
                        }
                    }
                },
                // Shutdown signal received
                _ = shutdown_signal.notified() => {
                    info!("Shutdown signal received, stopping read_task");
                    break;
                }
            }
        }
    }

    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        mut write: SplitSink<
            WebSocketStream<MaybeTlsStream<TcpStream>>,
            Message,
        >,
        mut receiver: UnboundedReceiver<ComInterfaceEvent>,
        state: Arc<Mutex<ComInterfaceStateWrapper>>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceEvent::SendBlock(block, _) => {
                    if let Err(e) = write.send(Message::Binary(block)).await {
                        // FIXME shall we retry?
                        error!("WebSocket write error: {e}");
                        state
                            .try_lock()
                            .unwrap()
                            .set(ComInterfaceState::Destroyed);
                        break;
                    }
                }
                ComInterfaceEvent::Destroy => break,
                _ => todo!(),
            }
        }
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
        InterfaceCreateError,
    > {
        let address = parse_url(&self.address).map_err(|_| {
            InterfaceCreateError::InvalidSetupData(
                "Invalid WebSocket URL".to_string(),
            )
        })?;
        if address.scheme() != "ws" && address.scheme() != "wss" {
            return Err(InterfaceCreateError::InvalidSetupData(
                "Invalid WebSocket URL scheme".to_string(),
            ));
        }
        info!("Connecting to WebSocket server at {address}");
        let (stream, _) = tokio_tungstenite::connect_async(address.clone())
            .await
            .map_err(|e| {
                error!("Failed to connect to WebSocket server: {e}");
                InterfaceCreateError::InterfaceError(
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

    fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> ComInterfaceAsyncFactoryResult {
        Box::pin(async move {
            self.create_interface(com_interface_proxy)
                .await
        })
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
