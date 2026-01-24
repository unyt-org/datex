use core::{cell::RefCell};
use datex_core::{
    global::{
        dxb_block::DXBBlock, protocol_structures::block_header::BlockType,
    },
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterfaceEvent,
            error::ComInterfaceError,
            factory::{
                ComInterfaceSyncFactory,
            },
            properties::{InterfaceDirection, InterfaceProperties},
            socket::ComInterfaceSocketUUID,
        },
    },
    task::{
        UnboundedReceiver, UnboundedSender,
        spawn_with_panic_notify_default,
    },
    values::core_values::endpoint::Endpoint,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    rc::Rc,
};
use std::sync::{Arc, Mutex};
use datex_core::network::com_interfaces::com_interface::ComInterfaceProxy;
use datex_core::network::com_interfaces::com_interface::socket_manager::ComInterfaceSocketManager;


impl MockupInterfaceSetupData {
    pub fn create_interface(
        mut self,
        proxy: ComInterfaceProxy,
    ) -> Result<InterfaceProperties, InterfaceCreateError> {
        let outgoing_queue = Rc::new(RefCell::new(Vec::new()));
        let (socket_uuid, sender) =
            self.create_and_add_socket(proxy.socket_manager)?;

        let name = self.name.clone();
        let direction = self.direction.clone();

        // setup event handler task
        spawn_with_panic_notify_default(Self::event_handler_task(
            outgoing_queue.clone(),
            self.sender_out.take(),
            proxy.event_receiver,
        ));
        if let Some(receiver) = self.receiver_in.take() {
            spawn_with_panic_notify_default(Self::send_incoming_blocks_task(
                receiver, sender,
            ));
        }

        Ok(InterfaceProperties {
            interface_type: "mockup".to_string(),
            channel: "mockup".to_string(),
            name: Some(name),
            direction,
            ..Default::default()
        })
    }

    fn create_and_add_socket(
        &self,
        socket_manager: Arc<Mutex<ComInterfaceSocketManager>>,
    ) -> Result<
        (ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>),
        ComInterfaceError,
    > {
        let direction = self.direction.clone();
        let (socket_uuid, sender) = socket_manager
            .lock()
            .unwrap()
            .create_and_init_socket_with_optional_endpoint(direction, 1, self.endpoint.clone());

        Ok((socket_uuid, sender))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MockupInterfaceSetupData {
    pub name: String,
    pub endpoint: Option<Endpoint>,
    pub direction: InterfaceDirection,

    #[serde(skip)]
    pub sender_out: Option<UnboundedSender<Vec<u8>>>,
    #[serde(skip)]
    pub receiver_in: Option<UnboundedReceiver<Vec<u8>>>,
}
impl Default for MockupInterfaceSetupData {
    fn default() -> Self {
        MockupInterfaceSetupData {
            name: "mockup".to_string(),
            endpoint: None,
            direction: InterfaceDirection::InOut,
            sender_out: None,
            receiver_in: None,
        }
    }
}

impl MockupInterfaceSetupData {
    pub fn new(name: &str) -> MockupInterfaceSetupData {
        MockupInterfaceSetupData {
            name: name.to_string(),
            endpoint: None,
            direction: InterfaceDirection::InOut,
            ..Default::default()
        }
    }
    pub fn new_with_direction(
        name: &str,
        direction: InterfaceDirection,
    ) -> MockupInterfaceSetupData {
        MockupInterfaceSetupData {
            name: name.to_string(),
            endpoint: None,
            direction,
            ..Default::default()
        }
    }
    pub fn new_with_endpoint(name: &str, endpoint: Endpoint) -> Self {
        MockupInterfaceSetupData {
            name: name.to_string(),
            endpoint: Some(endpoint),
            direction: InterfaceDirection::InOut,
            ..Default::default()
        }
    }
    pub fn new_with_endpoint_and_direction(
        name: &str,
        endpoint: Endpoint,
        direction: InterfaceDirection,
    ) -> Self {
        let mut setup_data = Self::new_with_endpoint(name, endpoint);
        setup_data.direction = direction;
        setup_data
    }

    async fn send_incoming_blocks_task(
        mut receiver: UnboundedReceiver<Vec<u8>>,
        mut sender: UnboundedSender<Vec<u8>>,
    ) {
        while let Some(block) = receiver.next().await {
            sender
                .send(block)
                .await
                .expect("Failed to send incoming block to MockupInterface");
        }
    }

    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        outgoing_queue: Rc<RefCell<Vec<(ComInterfaceSocketUUID, Vec<u8>)>>>,
        mut sender: Option<UnboundedSender<Vec<u8>>>,
        mut receiver: UnboundedReceiver<ComInterfaceEvent>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceEvent::SendBlock(block, socket_uuid) => {
                    let is_hello = block
                        .block_header
                        .flags_and_timestamp
                        .block_type() == BlockType::Hello;
                    let bytes = block.to_bytes();
                    if !is_hello {
                        outgoing_queue
                            .borrow_mut()
                            .push((socket_uuid, bytes.clone()));
                    }
                    if let Some(sender) = sender.as_mut() {
                        sender.start_send(bytes).expect(
                            "Failed to send outgoing block from MockupInterface",
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

impl ComInterfaceSyncFactory for MockupInterfaceSetupData {
    fn create_interface(
        self,
        proxy: ComInterfaceProxy,
    ) -> Result<InterfaceProperties, InterfaceCreateError>
    {
        self.create_interface(proxy)
    }

    fn get_default_properties() -> InterfaceProperties {
        InterfaceProperties {
            interface_type: "mockup".to_string(),
            channel: "mockup".to_string(),
            name: Some("mockup".to_string()),
            ..Default::default()
        }
    }
}