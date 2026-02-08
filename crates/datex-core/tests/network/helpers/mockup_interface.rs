use datex_core::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            factory::ComInterfaceSyncFactory,
            properties::{InterfaceDirection, ComInterfaceProperties},
        },
    },
    values::core_values::endpoint::Endpoint,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
};
use std::sync::{Arc, Mutex};
use datex_core::channel::mpsc::create_unbounded_channel;
use datex_core::global::dxb_block::DXBBlock;
use datex_core::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SendSuccess, SocketConfiguration, SocketProperties};

impl MockupInterfaceSetupData {
    pub fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let (sender, mut receiver) = create_unbounded_channel::<Vec<u8>>();
        let sender = Arc::new(Mutex::new(sender));

        Ok(ComInterfaceConfiguration::new_single_socket(
            ComInterfaceProperties {
                interface_type: "mockup".to_string(),
                channel: "mockup".to_string(),
                name: Some(self.name),
                direction: self.direction.clone(),
                ..Default::default()
            },
            SocketConfiguration::new(
                SocketProperties::new_with_maybe_direct_endpoint(self.direction, 1, self.endpoint),
                async gen move {
                    while let Some(block_bytes) = receiver.next().await {
                        yield Ok(block_bytes);
                    }
                },
                SendCallback::new_sync(move |block: DXBBlock| {
                    let bytes = block.to_bytes();
                    sender.lock().unwrap().start_send(bytes).expect(
                        "Failed to send outgoing block from MockupInterface",
                    );
                    Ok(SendSuccess::Sent)
                })
            )
        ))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MockupInterfaceSetupData {
    pub name: String,
    pub endpoint: Option<Endpoint>,
    pub direction: InterfaceDirection,
}
impl Default for MockupInterfaceSetupData {
    fn default() -> Self {
        MockupInterfaceSetupData {
            name: "mockup".to_string(),
            endpoint: None,
            direction: InterfaceDirection::InOut,
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
}

impl ComInterfaceSyncFactory for MockupInterfaceSetupData {
    fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        self.create_interface()
    }

    fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "mockup".to_string(),
            channel: "mockup".to_string(),
            name: Some("mockup".to_string()),
            ..Default::default()
        }
    }
}
