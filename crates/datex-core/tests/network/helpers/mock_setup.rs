use alloc::rc::Rc;
use core::str::FromStr;
use datex_core::{
    channel::mpsc::{
        UnboundedReceiver, UnboundedSender, create_unbounded_channel,
    },
    global::{
        dxb_block::{
            DXBBlock, IncomingEndpointContextSectionId, IncomingSection,
        },
        protocol_structures::block_header::FlagsAndTimestamp,
    },
    network::{
        com_hub::{ComHub, InterfacePriority},
        com_interfaces::com_interface::{
            ComInterfaceUUID,
            properties::{InterfaceDirection, ComInterfaceProperties},
            socket::ComInterfaceSocketUUID,
        },
    },
    runtime::{Runtime, RuntimeConfig},
    values::core_values::endpoint::Endpoint,
};
use core::cell::RefCell;
use log::{error, info};
use std::sync::{Once, mpsc};
use tokio::task::yield_now;

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
// 
// pub struct MockupSetupData {
//     pub local_endpoint: Endpoint,
//     pub interface_properties: ComInterfaceProperties,
//     pub com_hub_sections_sender: Option<UnboundedSender<IncomingSection>>,
//     pub interface_priority: InterfacePriority,
// }
// impl Default for MockupSetupData {
//     fn default() -> Self {
//         Self {
//             local_endpoint: TEST_ENDPOINT_ORIGIN.clone(),
//             interface_properties: ComInterfaceProperties::default(),
//             com_hub_sections_sender: None,
//             interface_priority: InterfacePriority::default(),
//         }
//     }
// }
// 
// impl MockupSetupData {
//     fn new_with_endpoint(endpoint: Endpoint) -> Self {
//         Self {
//             local_endpoint: endpoint,
//             ..Default::default()
//         }
//     }
// }
// 
// impl From<Endpoint> for MockupSetupData {
//     fn from(endpoint: Endpoint) -> Self {
//         Self::new_with_endpoint(endpoint)
//     }
// }
// 
// /// Helper function to create a mock setup with a com hub and a mockup interface
// pub async fn get_mock_setup_with_com_hub(
//     mock_setup_data: MockupSetupData,
// ) -> Rc<ComHub> {
//     // init mockup interface
//     let (proxy, interface) = ComInterfaceProxy::create_interface(
//         mock_setup_data.interface_properties,
//     );
// 
//     // init com hub
//     let com_hub = get_mock_setup_with_interface(
//         interface,
//         mock_setup_data.local_endpoint,
//         mock_setup_data.com_hub_sections_sender,
//         mock_setup_data.interface_priority,
//     );
// 
//     yield_now().await;
// 
//     (com_hub, proxy)
// }
// 
// /// Helper function to create a mock setup with a com hub and an existing interface
// pub fn get_mock_setup_with_interface(
//     interface: ComInterfaceWithReceivers,
//     local_endpoint: Endpoint,
//     incoming_sections_sender: Option<UnboundedSender<IncomingSection>>,
//     interface_priority: InterfacePriority,
// ) -> Rc<ComHub> {
//     // init com hub
//     let com_hub = ComHub::create(
//         local_endpoint,
//         incoming_sections_sender.unwrap_or_else(|| {
//             create_unbounded_channel::<IncomingSection>().0 // dummy sender
//         }),
//     );
// 
//     // add mockup interface to com_hub
//     com_hub
//         ._register_com_interface(interface, interface_priority)
//         .unwrap();
// 
//     com_hub
// }
// 
// /// Helper function to create a default mock setup with initialized channels for com hub and mockup interface
// pub async fn get_default_mock_setup_with_com_hub() -> (
//     Rc<ComHub>,
//     UnboundedReceiver<IncomingSection>,
// ) {
//     let (com_hub_sections_sender, com_hub_sections_receiver) =
//         create_unbounded_channel::<IncomingSection>();
// 
//     let (com_hub, proxy) = get_mock_setup_with_com_hub(MockupSetupData {
//         interface_properties: ComInterfaceProperties::default(),
//         com_hub_sections_sender: Some(com_hub_sections_sender),
//         ..Default::default()
//     })
//     .await;
// 
//     yield_now().await;
// 
//     (com_hub, com_hub_sections_receiver)
// }
// 
// /// Helper function to create a default mock setup with two com hubs connected to each other via mock interface channels
// pub async fn get_default_mock_setup_with_two_connected_com_hubs() -> (
//     (
//         Rc<ComHub>,
//         UnboundedReceiver<IncomingSection>,
//         ComInterfaceUUID,
//     ),
//     (
//         Rc<ComHub>,
//         UnboundedReceiver<IncomingSection>,
//         ComInterfaceUUID,
//     ),
// ) {
//     let (incoming_sections_sender_b, incoming_sections_receiver_b) =
//         create_unbounded_channel::<IncomingSection>();
//     let (incoming_sections_sender_a, incoming_sections_receiver_a) =
//         create_unbounded_channel::<IncomingSection>();
// 
//     let (com_hub_mut_a, interface_proxy_a) =
//         get_mock_setup_with_com_hub(MockupSetupData {
//             interface_properties: ComInterfaceProperties {
//                 name: Some("A->B".to_string()),
//                 channel: "mockup".to_string(),
//                 interface_type: "mockup".to_string(),
//                 ..Default::default()
//             },
//             local_endpoint: TEST_ENDPOINT_A.clone(),
//             com_hub_sections_sender: Some(incoming_sections_sender_a),
//             ..Default::default()
//         })
//         .await;
// 
//     let (com_hub_mut_b, interface_proxy_b) =
//         get_mock_setup_with_com_hub(MockupSetupData {
//             interface_properties: ComInterfaceProperties {
//                 name: Some("B->A".to_string()),
//                 channel: "mockup".to_string(),
//                 interface_type: "mockup".to_string(),
//                 ..Default::default()
//             },
//             local_endpoint: TEST_ENDPOINT_B.clone(),
//             com_hub_sections_sender: Some(incoming_sections_sender_b),
//             ..Default::default()
//         })
//         .await;
// 
//     let (interface_a_uuid, interface_b_uuid) =
//         ComInterfaceProxy::couple_bidirectional(
//             (interface_proxy_a, None),
//             (interface_proxy_b, None),
//         );
// 
//     (
//         (
//             com_hub_mut_a,
//             incoming_sections_receiver_a,
//             interface_a_uuid,
//         ),
//         (
//             com_hub_mut_b,
//             incoming_sections_receiver_b,
//             interface_b_uuid,
//         ),
//     )
// }
// 
// /// Helper function to create a mock setup with a full runtime and a mockup interface
// pub async fn get_mock_setup_with_runtime(
//     mock_setup_data: MockupSetupData,
// ) -> (Runtime, ComInterfaceProxy) {
//     // init com hub
//     let runtime = Runtime::create_native(RuntimeConfig::new_with_endpoint(
//         mock_setup_data.local_endpoint,
//     ))
//     .await;
// 
//     // init mockup interface
//     let (proxy, interface) = ComInterfaceProxy::create_interface(
//         mock_setup_data.interface_properties,
//     );
// 
//     // add mockup interface to com_hub
//     runtime
//         .com_hub()
//         ._register_com_interface(interface, mock_setup_data.interface_priority)
//         .unwrap();
//     (runtime, proxy)
// }
// 
// /// Helper function to create a default mock setup with two separate runtimes
// pub async fn get_mock_setup_default_with_two_connected_runtimes(
//     setup_data_a: MockupSetupData,
//     setup_data_b: MockupSetupData,
// ) -> (Runtime, Runtime) {
//     let (runtime_a, proxy_a) = get_mock_setup_with_runtime(setup_data_a).await;
// 
//     let (runtime_b, proxy_b) = get_mock_setup_with_runtime(setup_data_b).await;
// 
//     // couple interfaces
//     ComInterfaceProxy::couple_bidirectional((proxy_a, None), (proxy_b, None));
// 
//     (runtime_a, runtime_b)
// }
// 
// pub async fn send_block_with_body(
//     to: &[Endpoint],
//     body: &[u8],
//     com_hub: &Rc<ComHub>,
// ) -> DXBBlock {
//     let block = {
//         let mut block: DXBBlock = DXBBlock::default();
//         block.set_receivers(to);
//         block.body = body.to_vec();
//         com_hub.send_own_block_async(block.clone()).await.unwrap();
//         block
//     };
// 
//     yield_now().await;
//     block
// }
// 
// pub async fn send_empty_block(
//     to: &[Endpoint],
//     com_hub: &Rc<ComHub>,
// ) -> Result<DXBBlock, ()> {
//     let mut block: DXBBlock = DXBBlock::default();
//     block.set_receivers(to);
//     {
//         if let Ok(sent_block) = com_hub.send_own_block_async(block.clone()).await {
//             info!("Sent block: {:?}", sent_block);
//         } else {
//             error!("Failed to send block");
//             return Err(());
//         }
//     }
// 
//     Ok(block)
// }
// pub async fn get_next_received_single_block_from_receiver(
//     sections_receiver: &mut UnboundedReceiver<IncomingSection>,
// ) -> DXBBlock {
//     let section = sections_receiver.next().await.unwrap();
// 
//     match &section {
//         IncomingSection::SingleBlock((Some(block), id)) => {
//             // assert that endpoint context section id matches block
//             let block_id = block.get_block_id();
//             assert_eq!(
//                 IncomingEndpointContextSectionId::new(
//                     block_id.endpoint_context_id,
//                     block_id.current_section_index
//                 ),
//                 *id,
//                 "IncomingSection id does not match block id"
//             );
// 
//             block.clone()
//         }
//         _ => {
//             core::panic!("Expected single block, but got block stream");
//         }
//     }
// }
// 
// #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
// pub enum CollectedBlockType {
//     #[default]
//     All,
//     SingleBocks,
//     BlockStream,
// }
// 
// impl CollectedBlockType {
//     pub fn matches_section(&self, section: &IncomingSection) -> bool {
//         match self {
//             CollectedBlockType::SingleBocks => {
//                 matches!(section, IncomingSection::SingleBlock(_))
//             }
//             CollectedBlockType::BlockStream => {
//                 matches!(section, IncomingSection::BlockStream(_))
//             }
//             CollectedBlockType::All => true,
//         }
//     }
// }
// 
// pub async fn get_collected_received_blocks_from_receiver(
//     sections_receiver: &mut UnboundedReceiver<IncomingSection>,
//     collected_type: CollectedBlockType,
//     count: usize,
// ) -> Vec<DXBBlock> {
//     let mut blocks = vec![];
// 
//     let mut received_count = 0;
// 
//     while let Some(section) = sections_receiver.next().await {
//         if !collected_type.matches_section(&section) {
//             panic!(
//                 "Received section does not match collected block type {:?}",
//                 collected_type
//             );
//         }
// 
//         match section {
//             IncomingSection::SingleBlock((Some(block), ..)) => {
//                 blocks.push(block.clone());
//                 received_count += 1;
//                 info!("Received single block");
//             }
//             IncomingSection::BlockStream((Some(mut block_stream), ..)) => {
//                 info!("[START] block stream");
//                 while let Some(block) = block_stream.next().await {
//                     received_count += 1;
//                     blocks.push(block.clone());
//                     info!("Received block from stream");
// 
//                     if received_count >= count {
//                         break;
//                     }
//                 }
//                 info!("[END] receiving block stream");
//             }
//             _ => {
//                 panic!("Received section does not contain a block");
//             }
//         }
// 
//         if received_count >= count {
//             break;
//         }
//     }
// 
//     if blocks.len() != count {
//         panic!(
//             "Expected to receive {} blocks, but got {}",
//             count,
//             blocks.len()
//         );
//     }
// 
//     blocks
// }
// 
// pub async fn get_collected_outgoing_blocks_from_receiver(
//     event_receiver: &mut UnboundedReceiver<ComInterfaceEvent>,
//     count: usize,
// ) -> Vec<(DXBBlock, ComInterfaceSocketUUID)> {
//     let mut collected_blocks = vec![];
// 
//     let mut received_count = 0;
// 
//     while let Some(event) = event_receiver.next().await {
//         if let ComInterfaceEvent::SendBlock(block, socket_uuid) = event {
//             collected_blocks.push((block, socket_uuid));
//             received_count += 1;
// 
//             if received_count >= count {
//                 break;
//             }
//         }
//     }
// 
//     if collected_blocks.len() != count {
//         panic!(
//             "Expected to collect {} blocks, but got {}",
//             count,
//             collected_blocks.len()
//         );
//     }
// 
//     collected_blocks
// }
// 
// pub async fn get_next_outgoing_block_from_receiver(
//     event_receiver: &mut UnboundedReceiver<ComInterfaceEvent>,
// ) -> (DXBBlock, ComInterfaceSocketUUID) {
//     while let Some(event) = event_receiver.next().await {
//         if let ComInterfaceEvent::SendBlock(block, socket_uuid) = event {
//             return (block, socket_uuid);
//         }
//     }
// 
//     panic!("No outgoing block received");
// }
// 
// /// Helper function to send multiple blocks to a local mockup interface via its incoming blocks sender
// /// Changes the receivers of each block to TEST_ENDPOINT_ORIGIN before sending
// pub async fn send_multiple_blocks_to_local(
//     incoming_blocks_sender: &mut UnboundedSender<Vec<u8>>,
//     blocks: &mut Vec<DXBBlock>,
// ) {
//     for block in blocks.iter_mut() {
//         // set receiver to ORIGIN
//         block.set_receivers(vec![TEST_ENDPOINT_ORIGIN.clone()]);
//     }
// 
//     let block_bytes: Vec<Vec<u8>> =
//         blocks.iter().map(|block| block.to_bytes()).collect();
// 
//     for block in block_bytes.into_iter() {
//         incoming_blocks_sender.start_send(block).unwrap();
//         yield_now().await;
//     }
// }
