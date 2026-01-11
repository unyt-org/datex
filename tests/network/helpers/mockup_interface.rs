use core::{cell::RefCell, time::Duration};
use datex_core::{
    global::{
        dxb_block::DXBBlock, protocol_structures::block_header::BlockType,
    },
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterface, ComInterfaceImplEvent,
            error::ComInterfaceError,
            implementation::{
                ComInterfaceImplementation, ComInterfaceSyncFactory,
            },
            properties::{InterfaceDirection, InterfaceProperties},
            socket::ComInterfaceSocketUUID,
        },
    },
    task::{
        UnboundedReceiver, UnboundedSender, create_unbounded_channel,
        spawn_with_panic_notify, spawn_with_panic_notify_default,
    },
    utils::once_consumer::OnceConsumer,
    values::core_values::endpoint::Endpoint,
};
use datex_macros::{com_interface, create_opener};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Debug,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
};
use tokio::net::tcp::OwnedWriteHalf;

pub struct MockupInterface {
    pub(crate) outgoing_queue: Rc<RefCell<Vec<(ComInterfaceSocketUUID, Vec<u8>)>>>,
    com_interface: Rc<ComInterface>,
    setup_data: MockupInterfaceSetupData,
    pub socket_uuid: ComInterfaceSocketUUID,
}

impl Debug for MockupInterface {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MockupInterface")
            .field("outgoing_queue_length", &self.outgoing_queue.borrow().len())
            .field("setup_data", &self.setup_data)
            .finish()
    }
}

impl MockupInterface {
    pub fn new(
        mut setup_data: MockupInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
        let outgoing_queue = Rc::new(RefCell::new(Vec::new()));
        let (socket_uuid, sender) =
            Self::create_and_add_socket(&setup_data, com_interface.clone())?;

        let name = setup_data.name.clone();
        let direction = setup_data.direction.clone();

        // setup event handler task
        spawn_with_panic_notify_default(Self::event_handler_task(
            outgoing_queue.clone(),
            setup_data.sender_out.take(),
            com_interface.take_interface_impl_event_receiver(),
        ));
        if let Some(receiver) = setup_data.receiver_in.take() {
            spawn_with_panic_notify_default(Self::send_incoming_blocks_task(
                receiver, sender,
            ));
        }

        Ok((
            MockupInterface {
                socket_uuid,
                outgoing_queue,
                setup_data,
                com_interface: com_interface.clone(),
            },
            InterfaceProperties {
                interface_type: "mockup".to_string(),
                channel: "mockup".to_string(),
                name: Some(name),
                direction,
                ..Default::default()
            },
        ))
    }

    fn create_and_add_socket(
        setup_data: &MockupInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<
        (ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>),
        ComInterfaceError,
    > {
        let direction = setup_data.direction.clone();
        let (socket_uuid, sender) = com_interface
            .socket_manager()
            .lock()
            .unwrap()
            .create_and_init_socket(direction, 1);

        if let Some(endpoint) = &setup_data.endpoint {
            com_interface
                .socket_manager()
                .lock()
                .unwrap()
                .register_socket_with_endpoint(
                    socket_uuid.clone(),
                    endpoint.clone(),
                    1,
                )?;
        }
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
            ..Default::default()
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
}

impl ComInterfaceSyncFactory for MockupInterface {
    type SetupData = MockupInterfaceSetupData;

    fn create(
        setup_data: Self::SetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(MockupInterface, InterfaceProperties), InterfaceCreateError>
    {
        MockupInterface::new(setup_data, com_interface)
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

impl MockupInterface {
    pub fn last_block(&self) -> Option<Vec<u8>> {
        self.outgoing_queue
            .borrow()
            .last()
            .map(|(_, block)| block.clone())
    }
    pub fn last_socket_uuid(&self) -> Option<ComInterfaceSocketUUID> {
        self.outgoing_queue
            .borrow()
            .last()
            .map(|(socket_uuid, _)| socket_uuid.clone())
    }

    pub fn find_outgoing_block_for_socket(
        &self,
        socket_uuid: &ComInterfaceSocketUUID,
    ) -> Option<Vec<u8>> {
        self.outgoing_queue
            .borrow()
            .iter()
            .find(|(uuid, _)| uuid == socket_uuid)
            .map(|(_, block)| block.clone())
    }
    pub fn has_outgoing_block_for_socket(
        &self,
        socket_uuid: &ComInterfaceSocketUUID,
    ) -> bool {
        self.find_outgoing_block_for_socket(socket_uuid).is_some()
    }

    pub fn last_block_and_socket(
        &self,
    ) -> Option<(ComInterfaceSocketUUID, Vec<u8>)> {
        self.outgoing_queue.borrow().last().cloned()
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
        mut receiver: UnboundedReceiver<ComInterfaceImplEvent>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceImplEvent::SendBlock(block, socket_uuid) => {
                    let is_hello = {
                        match DXBBlock::from_bytes(&block).await {
                            Ok(block) => {
                                block
                                    .block_header
                                    .flags_and_timestamp
                                    .block_type()
                                    == BlockType::Hello
                            }
                            _ => false,
                        }
                    };
                    if !is_hello {
                        outgoing_queue
                            .borrow_mut()
                            .push((socket_uuid, block.clone()));
                    }
                    if let Some(sender) = sender.as_mut() {
                        sender.start_send(block).expect(
                            "Failed to send outgoing block from MockupInterface",
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

impl ComInterfaceImplementation for MockupInterface {}
