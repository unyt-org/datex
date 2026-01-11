use super::mockup_interface::{MockupInterface, MockupInterfaceSetupData};
use core::str::FromStr;
use datex_core::{
    global::dxb_block::{DXBBlock, IncomingSection},
    network::{
        block_handler::IncomingSectionsSinkType,
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
    pub setup_data: MockupInterfaceSetupData,
    pub priority: InterfacePriority,
    pub incoming_sections_sink_type: IncomingSectionsSinkType,
}
impl Default for MockupSetupData {
    fn default() -> Self {
        Self {
            local_endpoint: TEST_ENDPOINT_ORIGIN.clone(),
            setup_data: MockupInterfaceSetupData::default(),
            priority: InterfacePriority::default(),
            incoming_sections_sink_type: IncomingSectionsSinkType::Channel,
        }
    }
}

pub async fn get_mock_setup() -> (Rc<ComHub>, Rc<ComInterface>) {
    get_mock_setup_with_endpoint(
        TEST_ENDPOINT_ORIGIN.clone(),
        None,
        InterfaceDirection::InOut,
        InterfacePriority::default(),
        IncomingSectionsSinkType::Channel,
    )
    .await
}

pub async fn get_mock_setup_with_endpoint(
    endpoint: Endpoint,
    remote_endpoint: Option<Endpoint>,
    direction: InterfaceDirection,
    priority: InterfacePriority,
    sink_type: IncomingSectionsSinkType,
) -> (Rc<ComHub>, Rc<ComInterface>) {
    // init com hub
    let com_hub = ComHub::create(endpoint, AsyncContext::new(), sink_type);

    // init mockup interface
    let mockup_interface = ComInterface::create_sync_with_implementation::<
        MockupInterface,
    >(MockupInterfaceSetupData {
        endpoint: remote_endpoint,
        direction,
        ..Default::default()
    })
    .unwrap();

    // add mockup interface to com_hub
    com_hub.register_com_interface(mockup_interface.clone(), priority);

    (com_hub, mockup_interface.clone())
}

pub async fn get_runtime_with_mock_interface(
    setup: MockupSetupData,
) -> (Runtime, Rc<ComInterface>) {
    // init com hub
    let runtime = Runtime::init_native(RuntimeConfig::new_with_endpoint(
        setup.local_endpoint,
    ));

    // init mockup interface
    let mockup_interface_ref = ComInterface::create_sync_with_implementation::<
        MockupInterface,
    >(setup.setup_data)
    .unwrap();

    // add mockup interface to com_hub
    runtime
        .com_hub()
        .register_com_interface(mockup_interface_ref.clone(), setup.priority);
    (runtime, mockup_interface_ref)
}

pub async fn get_mock_setup_with_two_runtimes(
    setup_data_a: MockupSetupData,
    setup_data_b: MockupSetupData,
) -> (Runtime, Runtime) {
    let (runtime_a, _) = get_runtime_with_mock_interface(setup_data_a).await;

    let (runtime_b, _) = get_runtime_with_mock_interface(setup_data_b).await;

    (runtime_a, runtime_b)
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

pub async fn send_empty_block_and_update(
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

    yield_now().await;
    block
}

pub fn get_last_received_single_block_from_com_hub(
    com_hub: &ComHub,
) -> DXBBlock {
    let sections = com_hub.block_handler.drain_collected_sections();

    assert_eq!(sections.len(), 1);

    match &sections[0] {
        IncomingSection::SingleBlock((Some(block), ..)) => block.clone(),
        _ => {
            core::panic!("Expected single block, but got block stream");
        }
    }
}
pub fn get_all_received_single_blocks_from_com_hub(
    com_hub: &ComHub,
) -> Vec<DXBBlock> {
    let sections = com_hub.block_handler.drain_collected_sections();

    let mut blocks = vec![];

    for section in sections {
        match section {
            IncomingSection::SingleBlock((Some(block), ..)) => {
                blocks.push(block.clone());
            }
            _ => {
                core::panic!("Expected single block, but got block stream");
            }
        }
    }

    blocks
}
