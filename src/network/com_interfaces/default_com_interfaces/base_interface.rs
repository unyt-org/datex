use core::prelude::rust_2024::*;
use core::result::Result;
use std::collections::HashMap;

use crate::network::com_interfaces::com_interface::implementation::ComInterfaceImplementation;
use crate::network::com_interfaces::com_interface::state::{
    ComInterfaceState, ComInterfaceStateWrapper,
};
use crate::network::{
    com_hub::errors::ComHubError,
    com_interfaces::com_interface::properties::InterfaceDirection,
};

use crate::network::com_interfaces::com_interface::properties::InterfaceProperties;
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;
use crate::network::com_interfaces::com_interface::{
    ComInterface, ComInterfaceImplEvent, ComInterfaceInner,
};
use crate::stdlib::boxed::Box;
use crate::stdlib::cell::RefCell;
use crate::stdlib::pin::Pin;
use crate::stdlib::rc::Rc;
use crate::stdlib::string::String;
use crate::stdlib::vec::Vec;
use crate::values::core_values::endpoint::Endpoint;
use core::future::Future;
use log::error;

pub type OnSendCallback = dyn Fn(&[u8], ComInterfaceSocketUUID) -> Pin<Box<dyn Future<Output = bool>>>
    + 'static;

pub struct BaseInterface {
    com_interface: Rc<ComInterface>,
}

use crate::task::{UnboundedReceiver, UnboundedSender};
use datex_core::task::spawn_with_panic_notify_default;
use strum::Display;
use thiserror::Error;

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

pub struct BaseInterfaceHolder {
    sender: HashMap<ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>>,
    pub com_interface: Rc<ComInterface>,
}
impl BaseInterfaceHolder {
    pub fn new(setup_data: BaseInterfaceSetupData) -> BaseInterfaceHolder {
        // Create a headless ComInterface first
        let com_interface = Rc::new(ComInterface {
            inner: Rc::new(ComInterfaceInner::init(
                ComInterfaceState::NotConnected,
                setup_data.properties,
            )),
            implementation: RefCell::new(None),
        });

        // Create the implementation using the factory function
        let implementation = BaseInterface {
            com_interface: com_interface.clone(),
        };
        com_interface.set_implementation(Box::new(implementation));

        let interface_impl_event_receiver =
            com_interface.take_interface_impl_event_receiver();

        // todo: use async context
        spawn_with_panic_notify_default(Self::event_handler_task(
            setup_data.on_send_callback,
            interface_impl_event_receiver,
        ));

        BaseInterfaceHolder {
            sender: HashMap::new(),
            com_interface,
        }
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

    pub fn receive(
        &mut self,
        receiver_socket_uuid: ComInterfaceSocketUUID,
        data: Vec<u8>,
    ) -> Result<(), BaseInterfaceError> {
        if let Some(sender) = self.sender.get_mut(&receiver_socket_uuid) {
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
            .com_interface
            .socket_manager()
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
    ) -> (ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>) {
        let (socket_uuid, sender) = self.create_and_init_socket(direction);

        self.com_interface
            .socket_manager()
            .lock()
            .unwrap()
            .register_socket_with_endpoint(socket_uuid.clone(), endpoint, 1)
            .unwrap();
        (socket_uuid, sender)
    }
}

impl ComInterfaceImplementation for BaseInterface {}

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

#[cfg(test)]
mod tests {
    use crate::{
        network::com_interfaces::{
            com_interface::{
                properties::InterfaceProperties, state::ComInterfaceState,
            },
            default_com_interfaces::base_interface::{
                BaseInterfaceHolder, BaseInterfaceSetupData,
            },
        },
        utils::context::init_global_context,
    };
    use datex_core::run_async;

    #[tokio::test]
    pub async fn test_close() {
        run_async! {
            init_global_context();
            // Create a new interface
            let base_interface =
                BaseInterfaceHolder::new(BaseInterfaceSetupData::new(
                    InterfaceProperties::default(),
                    Box::new(|_, _| Box::pin(async move { true })),
                ))
                .com_interface
                .clone();
            assert_eq!(
                base_interface.current_state(),
                ComInterfaceState::NotConnected
            );
            assert!(base_interface.properties().close_timestamp.is_none());

            // Close the interface
            base_interface.close();
            assert_eq!(
                base_interface.current_state(),
                ComInterfaceState::NotConnected
            );
        }
    }
}
