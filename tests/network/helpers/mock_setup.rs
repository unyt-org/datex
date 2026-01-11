use super::mockup_interface::{MockupInterface, MockupInterfaceSetupData};
use core::str::FromStr;
use datex_core::{
    global::dxb_block::{DXBBlock, IncomingSection},
    network::{
        com_hub::{ComHub, InterfacePriority},
        com_interfaces::com_interface::{
            ComInterface, error::ComInterfaceError,
            properties::InterfaceDirection, socket::ComInterfaceSocketUUID,
        },
    },
    runtime::{AsyncContext, Runtime, RuntimeConfig},
    stdlib::{cell::RefCell, rc::Rc},
    task::{UnboundedReceiver, UnboundedSender},
    utils::once_consumer::OnceConsumer,
    values::core_values::endpoint::Endpoint,
};
use log::{error, info};
use std::{
    cell::RefMut,
    sync::{Once, mpsc},
};
use tokio::task::yield_now;
use webrtc::interceptor::mock;
use datex_core::global::dxb_block::IncomingEndpointContextSectionId;
use datex_core::task::create_unbounded_channel;

lazy_static::lazy_static! {
    pub static ref ANY : Endpoint = Endpoint::ANY.clone();

    pub static ref TEST_ENDPOINT_ORIGIN : Endpoint = Endpoint::from_str("@origin").unwrap();
    pub static ref TEST_ENDPOINT_A: Endpoint = Endpoint::from_str("@test-a").unwrap();
    pub static ref TEST_ENDPOINT_B: Endpoint = Endpoint::from_str("@test-b").unwrap();
    pub static ref TEST_ENDPOINT_C: Endpoint = Endpoint::from_str("@test-c").unwrap();
    pub static ref TEST_ENDPOINT_D: Endpoint = Endpoint::from_str("@test-d").unwrap();
    pub static ref TEST_ENDPOINT_E: Endpoint = Endpoint::from_str("@test-e").unwrap();
    pub static ref TEST_ENDPOINT_F: Endpoint = Endpoint::from_str("@test-f").unwrap();
    pub static ref TEST_ENDPOINT_G: Endpoint = Endpoint::from_str("@test-g").unwrap();
    pub static ref TEST_ENDPOINT_H: Endpoint = Endpoint::from_str("@test-h").unwrap();
    pub static ref TEST_ENDPOINT_I: Endpoint = Endpoint::from_str("@test-i").unwrap();
    pub static ref TEST_ENDPOINT_J: Endpoint = Endpoint::from_str("@test-j").unwrap();
    pub static ref TEST_ENDPOINT_K: Endpoint = Endpoint::from_str("@test-k").unwrap();
    pub static ref TEST_ENDPOINT_L: Endpoint = Endpoint::from_str("@test-l").unwrap();
    pub static ref TEST_ENDPOINT_M: Endpoint = Endpoint::from_str("@test-m").unwrap();
}

pub struct MockupSetupData {
    pub local_endpoint: Endpoint,
    pub interface_setup_data: MockupInterfaceSetupData,
    pub interface_priority: InterfacePriority,
    pub com_hub_sections_sender: Option<UnboundedSender<IncomingSection>>,
}
impl Default for MockupSetupData {
    fn default() -> Self {
        Self {
            local_endpoint: TEST_ENDPOINT_ORIGIN.clone(),
            interface_setup_data: MockupInterfaceSetupData::default(),
            interface_priority: InterfacePriority::default(),
            com_hub_sections_sender: None,
        }
    }
}

/// Helper function to create a mock setup with a com hub and a mockup interface
pub async fn get_mock_setup_with_com_hub(
    mock_setup_data: MockupSetupData,
) -> (Rc<ComHub>, Rc<ComInterface>) {
    // init mockup interface
    let mockup_interface = ComInterface::create_sync_with_implementation::<
        MockupInterface,
    >(mock_setup_data.interface_setup_data)
        .unwrap();

    // init com hub
    let com_hub = get_mock_setup_with_interface(
        mockup_interface.clone(),
        mock_setup_data.local_endpoint,
        mock_setup_data.com_hub_sections_sender,
        mock_setup_data.interface_priority,
    );

    (com_hub, mockup_interface.clone())
}

/// Helper function to create a default mock setup with two com hubs connected to each other via mock interface channels
pub async fn get_default_mock_setup_with_two_connected_com_hubs() -> (
    (
        Rc<ComHub>,
        Rc<ComInterface>,
        UnboundedReceiver<IncomingSection>,
    ),
    (
        Rc<ComHub>,
        Rc<ComInterface>,
        UnboundedReceiver<IncomingSection>,
    )
) {
    let (sender_a, receiver_a) = create_unbounded_channel::<Vec<u8>>();
    let (sender_b, receiver_b) = create_unbounded_channel::<Vec<u8>>();

    let (incoming_sections_sender_b, incoming_sections_receiver_b) = create_unbounded_channel::<IncomingSection>();
    let (incoming_sections_sender_a, incoming_sections_receiver_a) = create_unbounded_channel::<IncomingSection>();

    let (com_hub_mut_a, com_interface_a) = get_mock_setup_with_com_hub(MockupSetupData {
        interface_setup_data: MockupInterfaceSetupData {
            receiver_in: Some(receiver_b),
            sender_out: Some(sender_a),
            ..Default::default()
        },
        local_endpoint: TEST_ENDPOINT_A.clone(),
        com_hub_sections_sender: Some(incoming_sections_sender_a),
        ..Default::default()
    }).await;

    let (com_hub_mut_b, com_interface_b) = get_mock_setup_with_com_hub(MockupSetupData {
        interface_setup_data: MockupInterfaceSetupData {
            receiver_in: Some(receiver_a),
            sender_out: Some(sender_b),
            ..Default::default()
        },
        local_endpoint: TEST_ENDPOINT_B.clone(),
        com_hub_sections_sender: Some(incoming_sections_sender_b),
        ..Default::default()
    }).await;

    (
        (
            com_hub_mut_a,
            com_interface_a,
            incoming_sections_receiver_a
        ),
        (
            com_hub_mut_b,
            com_interface_b,
            incoming_sections_receiver_b
        )
    )
}

/// Helper function to create a mock setup with a com hub and an existing interface
pub fn get_mock_setup_with_interface(
    interface: Rc<ComInterface>,
    local_endpoint: Endpoint,
    incoming_sections_sender: Option<UnboundedSender<IncomingSection>>,
    interface_priority: InterfacePriority,
) -> Rc<ComHub> {
    // init com hub
    let com_hub = ComHub::create(
        local_endpoint,
        incoming_sections_sender.unwrap_or_else(|| {
            create_unbounded_channel::<IncomingSection>().0 // dummy sender
        }),
        AsyncContext::new(),
    );

    // add mockup interface to com_hub
    com_hub
        .register_com_interface(interface, interface_priority)
        .unwrap();

    com_hub
}


/// Helper function to create a default mock setup with initialized channels for com hub and mockup interface
pub async fn get_default_mock_setup_with_com_hub() -> (
    Rc<ComHub>,
    Rc<ComInterface>,
    UnboundedSender<Vec<u8>>,
    UnboundedReceiver<IncomingSection>,
) {
    let (interface_in_sender, interface_in_receiver) = create_unbounded_channel::<Vec<u8>>();
    let (com_hub_sections_sender, com_hub_sections_receiver) = create_unbounded_channel::<IncomingSection>();

    let (com_hub, com_interface) = get_mock_setup_with_com_hub(MockupSetupData {
        interface_setup_data: MockupInterfaceSetupData {
            receiver_in: Some(interface_in_receiver),
            endpoint: Some(TEST_ENDPOINT_B.clone()),
            ..Default::default()
        },
        com_hub_sections_sender: Some(com_hub_sections_sender),
        ..Default::default()
    }).await;

    (
        com_hub,
        com_interface,
        interface_in_sender,
        com_hub_sections_receiver,
    )
}


/// Helper function to create a mock setup with a full runtime and a mockup interface
pub async fn get_mock_setup_with_runtime(
    mock_setup_data: MockupSetupData,
) -> (Runtime, Rc<ComInterface>) {
    // init com hub
    let runtime = Runtime::init_native(RuntimeConfig::new_with_endpoint(
        mock_setup_data.local_endpoint,
    ));

    // init mockup interface
    let mockup_interface_ref = ComInterface::create_sync_with_implementation::<
        MockupInterface,
    >(mock_setup_data.interface_setup_data)
    .unwrap();

    // add mockup interface to com_hub
    runtime
        .com_hub()
        .register_com_interface(mockup_interface_ref.clone(), mock_setup_data.interface_priority)
        .unwrap();
    (runtime, mockup_interface_ref)
}

/// Helper function to create a default mock setup with two separate runtimes
pub async fn get_mock_setup_with_two_runtimes(
    setup_data_a: MockupSetupData,
    setup_data_b: MockupSetupData,
) -> (Runtime, Runtime) {
    let (runtime_a, _) = get_mock_setup_with_runtime(setup_data_a).await;

    let (runtime_b, _) = get_mock_setup_with_runtime(setup_data_b).await;

    (runtime_a, runtime_b)
}

/// Helper function to create a default mock setup with two connected runtimes via a mockup interface channel
pub async fn get_mock_setup_default_with_two_connected_runtimes(
    endpoint_a: Endpoint,
    endpoint_b: Endpoint,
) -> (Runtime, Runtime) {
    let (sender_a, receiver_a) = create_unbounded_channel::<Vec<u8>>();
    let (sender_b, receiver_b) = create_unbounded_channel::<Vec<u8>>();

    get_mock_setup_with_two_runtimes(
        MockupSetupData {
            local_endpoint: endpoint_a,
            interface_setup_data: MockupInterfaceSetupData {
                receiver_in: Some(receiver_b),
                sender_out: Some(sender_a),
                ..Default::default()
            },
            ..Default::default()
        },
        MockupSetupData {
            local_endpoint: endpoint_b,
            interface_setup_data: MockupInterfaceSetupData {
                receiver_in: Some(receiver_a),
                sender_out: Some(sender_b),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .await
}

pub async fn send_block_with_body(
    to: &[Endpoint],
    body: &[u8],
    com_hub: &Rc<ComHub>,
) -> DXBBlock {
    let block = {
        let mut block: DXBBlock = DXBBlock::default();
        block.set_receivers(to);
        block.body = body.to_vec();
        com_hub.send_own_block(block.clone()).await.unwrap();
        block
    };

    yield_now().await;
    block
}

pub async fn send_empty_block(
    to: &[Endpoint],
    com_hub: &Rc<ComHub>,
) -> DXBBlock {
    let mut block: DXBBlock = DXBBlock::default();
    block.set_receivers(to);
    {
        if let Ok(sent_block) = com_hub.send_own_block(block.clone()).await {
            info!("Sent block: {:?}", sent_block);
        } else {
            error!("Failed to send block");
        }
    }

    block
}
pub async fn get_last_received_single_block_from_receiver(
    sections_receiver: &mut UnboundedReceiver<IncomingSection>
) -> DXBBlock {
    let section = sections_receiver.next().await.unwrap();

    match &section {
        IncomingSection::SingleBlock((Some(block), id)) => {
            // assert that endpoint context section id matches block
            let block_id = block.get_block_id();
            assert_eq!(
                IncomingEndpointContextSectionId::new(
                    block_id.endpoint_context_id,
                    block_id.current_section_index
                ),
                *id,
                "IncomingSection id does not match block id"
            );

            block.clone()
        }
        _ => {
            core::panic!("Expected single block, but got block stream");
        }
    }
}
pub async fn get_collected_received_single_blocks_from_receiver(
    sections_receiver: &mut UnboundedReceiver<IncomingSection>,
    count: usize,
) -> Vec<DXBBlock> {
    let mut blocks = vec![];

    for (received_count, section) in sections_receiver.next().await.into_iter().enumerate() {
        if received_count >= count {
            break;
        }
        match section {
            IncomingSection::SingleBlock((Some(block), ..)) => {
                blocks.push(block.clone());
            }
            _ => {
                core::panic!("Expected single block, but got block stream");
            }
        }
    }
    
    if blocks.len() != count {
        panic!("Expected to receive {} blocks, but got {}", count, blocks.len());
    }

    blocks
}
