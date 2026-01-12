use core::{prelude::rust_2024::*, result::Result};
use std::collections::HashMap;

use crate::network::{
    com_hub::errors::ComHubError,
    com_interfaces::com_interface::{
        properties::InterfaceDirection,
    },
};

use crate::{
    network::com_interfaces::com_interface::{
        ComInterface, ComInterfaceImplEvent,
        properties::InterfaceProperties, socket::ComInterfaceSocketUUID,
    },
    stdlib::{
        boxed::Box, pin::Pin, string::String, vec::Vec,
    },
    values::core_values::endpoint::Endpoint,
};
use core::future::Future;
use crate::stdlib::sync::{Arc, Mutex};
use log::error;
use crate::task::{UnboundedReceiver, UnboundedSender};
use datex_core::task::spawn_with_panic_notify_default;
use strum::Display;
use thiserror::Error;
use crate::network::com_interfaces::com_interface::{ComInterfaceProxy, ComInterfaceUUID};
use crate::network::com_interfaces::com_interface::socket_manager::ComInterfaceSocketManager;
use crate::network::com_interfaces::com_interface::state::ComInterfaceStateWrapper;

pub type OnSendCallback = dyn Fn(&[u8], ComInterfaceSocketUUID) -> Pin<Box<dyn Future<Output = bool>>>
+ 'static;

#[derive(Debug, Display, Error)]
pub enum BaseInterfaceError {
    SendError,
    ReceiveError,
    SocketNotFound,
    InterfaceNotFound,
    InvalidInput(String),
    ComHubError(ComHubError),
}

impl From<ComHubError> for BaseInterfaceError {
    fn from(err: ComHubError) -> Self {
        BaseInterfaceError::ComHubError(err)
    }
}

pub struct BaseInterface {
    pub uuid: ComInterfaceUUID,
    pub state: Arc<Mutex<ComInterfaceStateWrapper>>,
    pub socket_manager: Arc<Mutex<ComInterfaceSocketManager>>,
    senders: HashMap<ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>>,
}
impl BaseInterface {
    pub fn create(setup_data: BaseInterfaceSetupData) -> (BaseInterface, ComInterface) {

        let (proxy, interface) = ComInterfaceProxy::create_interface(
            setup_data.properties.clone(),
        );

        // todo: use async context
        spawn_with_panic_notify_default(Self::event_handler_task(
            setup_data.on_send_callback,
            proxy.event_receiver,
        ));

        (
            BaseInterface {
                uuid: proxy.uuid,
                state: proxy.state,
                socket_manager: proxy.socket_manager,
                senders: HashMap::new(),
            },
            interface,
        )
    }

    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        on_send_callback: Box<OnSendCallback>,
        mut receiver: UnboundedReceiver<ComInterfaceImplEvent>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceImplEvent::SendBlock(block, socket_uuid) => {
                    if !on_send_callback(&block, socket_uuid).await {
                        error!("BaseInterface send error");
                        // todo: handle error
                    }
                }
                _ => todo!(),
            }
        }
    }

    pub fn send_incoming_data(
        &mut self,
        receiver_socket_uuid: ComInterfaceSocketUUID,
        data: Vec<u8>,
    ) -> Result<(), BaseInterfaceError> {
        if let Some(sender) = self.senders.get_mut(&receiver_socket_uuid) {
            sender
                .start_send(data)
                .map_err(|_| BaseInterfaceError::ReceiveError)?;
            Ok(())
        } else {
            Err(BaseInterfaceError::SocketNotFound)
        }
    }

    fn create_and_init_socket(
        &mut self,
        direction: InterfaceDirection,
    ) -> (ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>) {
        let (uuid, sender) = self
            .socket_manager
            .lock()
            .unwrap()
            .create_and_init_socket(direction, 1);
        (uuid, sender)
    }

    /// Registers and initializes a new socket with the given endpoint and direction
    /// Returns the socket UUID and a sender to send data to the socket
    pub fn register_new_socket_with_endpoint(
        &mut self,
        direction: InterfaceDirection,
        endpoint: Endpoint,
    ) -> ComInterfaceSocketUUID {
        let (socket_uuid, sender) = self.create_and_init_socket(direction);

        self
            .socket_manager
            .lock()
            .unwrap()
            .register_socket_with_endpoint(socket_uuid.clone(), endpoint, 1)
            .unwrap();

        self.senders.insert(socket_uuid.clone(), sender.clone());

        socket_uuid
    }
}

#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct BaseInterfaceSetupData {
    pub properties: InterfaceProperties,
    pub on_send_callback: Box<OnSendCallback>,
}

impl BaseInterfaceSetupData {
    pub fn new(
        properties: InterfaceProperties,
        on_send_callback: Box<OnSendCallback>,
    ) -> Self {
        BaseInterfaceSetupData {
            properties,
            on_send_callback,
        }
    }
    pub fn with_callback(on_send_callback: Box<OnSendCallback>) -> Self {
        BaseInterfaceSetupData {
            properties: InterfaceProperties::default(),
            on_send_callback,
        }
    }
}