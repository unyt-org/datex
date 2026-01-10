use crate::stdlib::cell::RefCell;
use crate::stdlib::rc::Rc;
use crate::stdlib::{future::Future, pin::Pin, time::Duration};
use core::prelude::rust_2024::*;
use core::result::Result;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use log::{error, info};
use tokio::net::TcpStream;
use tungstenite::Message;
use url::Url;

use super::websocket_common::{
    WebSocketClientInterfaceSetupData, parse_url,
};
use crate::network::com_hub::errors::InterfaceCreateError;
use crate::network::com_interfaces::com_interface::ComInterface;
use crate::network::com_interfaces::com_interface::error::ComInterfaceError;
use crate::network::com_interfaces::com_interface::implementation::ComInterfaceImplementation;
use crate::network::com_interfaces::com_interface::implementation::{
    ComInterfaceAsyncFactory, ComInterfaceSyncFactory,
};
use crate::network::com_interfaces::com_interface::properties::{
    InterfaceDirection, InterfaceProperties,
};
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;
use crate::network::com_interfaces::com_interface::state::ComInterfaceState;
use crate::task::spawn_with_panic_notify_default;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub struct WebSocketClientNativeInterface {
    pub address: Url,
    pub socket_uuid: ComInterfaceSocketUUID,
    websocket_stream:
        RefCell<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    com_interface: Rc<ComInterface>,
}
impl WebSocketClientNativeInterface {
    async fn create(
        setup_data: WebSocketClientInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
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
        let (write, mut read) = stream.split();

        let (socket_uuid, mut sender) = com_interface
            .socket_manager()
            .lock()
            .unwrap()
            .create_and_init_socket(InterfaceDirection::InOut, 1);

        let state = com_interface.state();

        spawn_with_panic_notify_default(async move {
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
                        state
                            .try_lock()
                            .unwrap()
                            .set(ComInterfaceState::Destroyed);
                        break;
                    }
                }
            }
        });

        Ok((
            WebSocketClientNativeInterface {
                address: address.clone(),
                socket_uuid,
                com_interface,
                websocket_stream: RefCell::new(write),
            },
            InterfaceProperties {
                name: Some(address.to_string()),
                ..Self::get_default_properties()
            },
        ))
    }
}

impl ComInterfaceAsyncFactory for WebSocketClientNativeInterface {
    type SetupData = WebSocketClientInterfaceSetupData;

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
            >,
        >,
    > {
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

impl ComInterfaceImplementation for WebSocketClientNativeInterface {
    fn send_block<'a>(
        &'a self,
        block: &'a [u8],
        _: ComInterfaceSocketUUID,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        Box::pin(async move {
            // TODO: no borrow across await
            let mut websocket_stream = self.websocket_stream.borrow_mut();
            websocket_stream
                .send(Message::Binary(block.to_vec()))
                .await
                .map_err(|e| {
                    error!("Error sending message: {e:?}");
                    false
                })
                .is_ok()
        })
    }

    fn handle_destroy<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        todo!("#210")
    }

    fn handle_reconnect<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        todo!()
    }
}
