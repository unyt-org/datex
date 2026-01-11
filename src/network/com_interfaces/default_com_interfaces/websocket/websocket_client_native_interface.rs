use crate::stdlib::rc::Rc;
use crate::stdlib::sync::{Arc, Mutex};
use crate::stdlib::time::Duration;
use core::prelude::rust_2024::*;
use core::result::Result;
use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use log::{error, info};
use tokio::net::TcpStream;
use tungstenite::Message;
use url::Url;

use super::websocket_common::{WebSocketClientInterfaceSetupData, parse_url};
use crate::network::com_hub::errors::InterfaceCreateError;
use crate::network::com_interfaces::com_interface::error::ComInterfaceError;
use crate::network::com_interfaces::com_interface::implementation::{
    ComInterfaceAsyncFactory, ComInterfaceSyncFactory,
};
use crate::network::com_interfaces::com_interface::implementation::{
    ComInterfaceAsyncFactoryResult, ComInterfaceImplementation,
};
use crate::network::com_interfaces::com_interface::properties::{
    InterfaceDirection, InterfaceProperties,
};
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;
use crate::network::com_interfaces::com_interface::state::{
    ComInterfaceState, ComInterfaceStateWrapper,
};
use crate::network::com_interfaces::com_interface::{
    ComInterface, ComInterfaceImplEvent,
};
use crate::task::{
    UnboundedReceiver, UnboundedSender, spawn_with_panic_notify_default,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub struct WebSocketClientNativeInterface {
    pub address: Url,
    pub socket_uuid: ComInterfaceSocketUUID,
    com_interface: Rc<ComInterface>,
}
impl WebSocketClientNativeInterface {
    async fn create(
        setup_data: WebSocketClientInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
        let (address, write, read) =
            Self::create_websocket_client_connection(&setup_data).await?;

        let (socket_uuid, sender) = com_interface
            .socket_manager()
            .lock()
            .unwrap()
            .create_and_init_socket(InterfaceDirection::InOut, 1);

        let state = com_interface.state();
        let interface_impl_event_receiver =
            com_interface.take_interface_impl_event_receiver();

        spawn_with_panic_notify_default(Self::read_task(
            read,
            sender,
            state.clone(),
        ));

        spawn_with_panic_notify_default(Self::event_handler_task(
            write,
            interface_impl_event_receiver,
            state,
        ));

        Ok((
            WebSocketClientNativeInterface {
                address: address.clone(),
                socket_uuid,
                com_interface,
            },
            InterfaceProperties {
                name: Some(address.to_string()),
                ..Self::get_default_properties()
            },
        ))
    }

    /// background task to read messages from the websocket
    async fn read_task(
        mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        mut sender: UnboundedSender<Vec<u8>>,
        state: Arc<Mutex<ComInterfaceStateWrapper>>,
    ) {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    sender.start_send(data).unwrap();
                }
                Ok(_) => {
                    error!("Invalid message type received");
                }
                Err(e) => {
                    error!("WebSocket read error: {e}");
                    state.try_lock().unwrap().set(ComInterfaceState::Destroyed);
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
        mut receiver: UnboundedReceiver<ComInterfaceImplEvent>,
        state: Arc<Mutex<ComInterfaceStateWrapper>>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceImplEvent::SendBlock(block, _) => {
                    if let Err(e) = write.send(Message::Binary(block)).await {
                        error!("WebSocket write error: {e}");
                        state
                            .try_lock()
                            .unwrap()
                            .set(ComInterfaceState::Destroyed);
                        break;
                    }
                }
                _ => todo!(),
            }
        }
    }

    /// initialize a new websocket client connection
    async fn create_websocket_client_connection(
        setup_data: &WebSocketClientInterfaceSetupData,
    ) -> Result<
        (
            Url,
            SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
            SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        ),
        InterfaceCreateError,
    > {
        let address = parse_url(&setup_data.address).map_err(|_| {
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

impl ComInterfaceImplementation for WebSocketClientNativeInterface {}

impl ComInterfaceAsyncFactory for WebSocketClientNativeInterface {
    type SetupData = WebSocketClientInterfaceSetupData;

    fn create(
        setup_data: Self::SetupData,
        com_interface: Rc<ComInterface>,
    ) -> ComInterfaceAsyncFactoryResult<Self> {
        Box::pin(async move {
            WebSocketClientNativeInterface::create(setup_data, com_interface)
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
