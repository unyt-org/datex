use crate::{
    channel::mpsc::{
        UnboundedReceiver, UnboundedSender, create_unbounded_channel,
    },
    collections::HashMap,
    global::dxb_block::{
        BlockId, DXBBlock, IncomingBlockNumber, IncomingContextId,
        IncomingEndpointContextId, IncomingEndpointContextSectionId,
        IncomingSection, IncomingSectionIndex, OutgoingContextId,
        OutgoingSectionIndex,
    },
    network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID,
    std_random::RandomState,
    stdlib::{boxed::Box, collections::BTreeMap, rc::Rc, vec, vec::Vec},
    utils::time::Time,
};
use core::{cell::RefCell, fmt::Debug, prelude::rust_2024::*};
use log::info;
use ringmap::RingMap;

// TODO #170: store scope memory
#[derive(Debug)]
pub struct ScopeContext {
    pub next_section_index: IncomingSectionIndex,
    pub next_block_number: IncomingBlockNumber,
    /// timestamp of the last keep alive block
    /// when a specific time has passed since the timestamp, the scope context is disposed
    /// TODO #171: implement dispose of scope context
    pub keep_alive_timestamp: u64,
    // a reference to the sender for the current section
    pub current_queue_sender: Option<UnboundedSender<DXBBlock>>,
    // a cache for all blocks indexed by their block number
    pub cached_blocks: BTreeMap<IncomingBlockNumber, DXBBlock>,
}

/// A scope context storing scopes of incoming DXB blocks
impl Default for ScopeContext {
    fn default() -> Self {
        ScopeContext {
            next_section_index: 0,
            next_block_number: 0,
            keep_alive_timestamp: Time::now(),
            current_queue_sender: None,
            cached_blocks: BTreeMap::new(),
        }
    }
}

// fn that gets a scope context as callback
type SectionObserver = Box<dyn FnMut(IncomingSection)>;

#[derive(Clone, Debug)]
pub struct BlockHistoryData {
    /// if block originated from local endpoint, the socket uuid is None,
    /// otherwise it is the uuid of the incoming socket
    pub original_socket_uuid: Option<ComInterfaceSocketUUID>,
}

pub struct BlockHandler {
    pub current_context_id: RefCell<OutgoingContextId>,

    /// a map of active request scopes for incoming blocks
    pub block_cache: RefCell<HashMap<IncomingEndpointContextId, ScopeContext>>,

    /// a queue of incoming request scopes
    /// the scopes can be retrieved from the request_scopes map
    pub incoming_sections_sender: RefCell<UnboundedSender<IncomingSection>>,

    /// a map of observers for incoming response blocks (by context_id + block_index)
    /// contains an observer callback and an optional queue of blocks if the response block is a multi-block stream
    pub section_observers: RefCell<
        HashMap<(IncomingContextId, IncomingSectionIndex), SectionObserver>,
    >,

    /// history of all incoming blocks
    pub incoming_blocks_history:
        RefCell<RingMap<BlockId, BlockHistoryData, RandomState>>,
}

impl Debug for BlockHandler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BlockHandler")
            .field("current_context_id", &self.current_context_id)
            .field("block_cache", &self.block_cache)
            .field("incoming_blocks_history", &self.incoming_blocks_history)
            .finish()
    }
}

const RING_MAP_CAPACITY: usize = 500;

impl BlockHandler {
    pub fn init(
        incoming_sections_sender: UnboundedSender<IncomingSection>,
    ) -> BlockHandler {
        BlockHandler {
            current_context_id: RefCell::new(0),
            block_cache: RefCell::new(HashMap::new()),
            incoming_sections_sender: RefCell::new(incoming_sections_sender),
            section_observers: RefCell::new(HashMap::new()),
            incoming_blocks_history: RefCell::new(
                RingMap::with_capacity_and_hasher(
                    RING_MAP_CAPACITY,
                    RandomState::default(),
                ),
            ),
        }
    }

    /// Adds a block to the history of incoming blocks
    /// if the block is not already in the history
    /// returns true if the block was added and not already in the history
    pub fn add_block_id_to_history(
        &self,
        block_id: BlockId,
        original_socket_uuid: Option<ComInterfaceSocketUUID>,
    ) {
        let mut history = self.incoming_blocks_history.borrow_mut();
        // only add if original block
        if !history.contains_key(&block_id) {
            let block_data = BlockHistoryData {
                original_socket_uuid,
            };
            history.insert(block_id, block_data);
        }
    }

    /// Checks if a block is already in the history
    pub fn is_block_in_history(&self, block: &DXBBlock) -> bool {
        let history = self.incoming_blocks_history.borrow();
        let block_id = block.get_block_id();
        history.contains_key(&block_id)
    }

    pub fn get_block_data_from_history(
        &self,
        block: &DXBBlock,
    ) -> Option<BlockHistoryData> {
        let history = self.incoming_blocks_history.borrow();
        let block_id = block.get_block_id();
        history.get(&block_id).cloned()
    }

    /// Handles an incoming block by either putting it into the request queue
    /// or calling the observer for the block if it is a response block
    pub fn handle_incoming_block(&self, block: DXBBlock) {
        info!("Handling incoming block...");
        let context_id = block.block_header.context_id;
        let section_index = block.block_header.section_index;
        let block_number = block.block_header.block_number;
        let is_response = block
            .block_header
            .flags_and_timestamp
            .block_type()
            .is_response();

        info!(
            "Received block (context={context_id}, section={section_index}, block_nr={block_number})"
        );

        // handle observers if response block
        if is_response {
            self.handle_incoming_response_block(block);
        } else {
            self.handle_incoming_request_block(block);
        }
    }

    // Handles incoming request blocks by putting them into the request queue
    fn handle_incoming_request_block(&self, block: DXBBlock) {
        let new_sections =
            self.extract_complete_sections_with_new_incoming_block(block);
        // put into request queue
        let sender = &self.incoming_sections_sender;
        for section in new_sections {
            sender.borrow_mut().start_send(section).unwrap();
        }
    }

    /// Handles incoming response blocks by calling the observer if an observer is registered
    /// Returns true when the observer has consumed all blocks and should be removed
    fn handle_incoming_response_block(&self, block: DXBBlock) {
        let context_id = block.block_header.context_id;
        let endpoint_context_id = IncomingEndpointContextId {
            sender: block.routing_header.sender.clone(),
            context_id,
        };
        let new_sections =
            self.extract_complete_sections_with_new_incoming_block(block);
        // try to call the observer for the incoming response block
        for section in new_sections {
            let section_index = section.get_section_index();

            if let Some(observer) = self
                .section_observers
                .borrow_mut()
                .get_mut(&(context_id, section_index))
            {
                // call the observer with the new section
                observer(section);
            } else {
                // no observer for this scope id + block index
                log::warn!(
                    "No observer for incoming response block (scope={endpoint_context_id:?}, block={section_index}), dropping block"
                );
            };
        }
    }

    /// Takes a new incoming block and returns a vector of all new available incoming sections
    /// for the block's scope
    fn extract_complete_sections_with_new_incoming_block(
        &self,
        block: DXBBlock,
    ) -> Vec<IncomingSection> {
        let section_index = block.block_header.section_index;
        let block_number = block.block_header.block_number;
        let is_end_of_section =
            block.block_header.flags_and_timestamp.is_end_of_section();
        let is_end_of_context =
            block.block_header.flags_and_timestamp.is_end_of_context();
        let endpoint_context_id = IncomingEndpointContextId {
            sender: block.routing_header.sender.clone(),
            context_id: block.block_header.context_id,
        };
        let section_context_id = IncomingEndpointContextSectionId::new(
            endpoint_context_id.clone(),
            section_index,
        );

        // get scope context if it already exists
        let has_scope_context =
            self.block_cache.borrow().contains_key(&endpoint_context_id);

        // Case 1: shortcut if no scope context exists and the block is a single block
        if !has_scope_context
            && block_number == 0
            && (is_end_of_section || is_end_of_context)
        {
            return vec![IncomingSection::SingleBlock((
                Some(block),
                section_context_id.clone(),
            ))];
        }

        // make sure a scope context exists from here on
        let mut request_scopes = self.block_cache.borrow_mut();
        let scope_context = request_scopes
            .entry(endpoint_context_id.clone())
            .or_default();

        // TODO #172: what happens if the endpoint has not received all blocks starting with block_number 0?
        // we should still potentially process those blocks

        // Case 2: if the block is the next expected block in the current section, put it into the
        // section block queue and try to drain blocks from the cache
        if block_number == scope_context.next_block_number {
            // list of IncomingSections that is returned at the end
            let mut new_blocks = vec![];

            // initial values for loop variables from input block
            let mut is_end_of_context = is_end_of_context;
            let mut is_end_of_section = is_end_of_section;
            let mut next_block = block;
            let mut section_index = section_index;

            // loop over the input block and potential blocks from the cache until the next block cannot be found
            // or the end of the scope is reached
            loop {
                if let Some(sender) = &mut scope_context.current_queue_sender {
                    // send the next block to the section queue receiver
                    sender.start_send(next_block).expect(
                        "Failed to send block to current section queue",
                    );
                } else {
                    // create a new block queue for the current section
                    let (mut sender, receiver) = create_unbounded_channel();

                    // add the first block to the queue
                    new_blocks.push(IncomingSection::BlockStream((
                        Some(receiver),
                        IncomingEndpointContextSectionId::new(
                            endpoint_context_id.clone(),
                            section_index,
                        ),
                    )));

                    // send the next block to the section queue receiver
                    sender.start_send(next_block).expect(
                        "Failed to send first block to current section queue",
                    );

                    scope_context.current_queue_sender = Some(sender);
                }

                // cleanup / prepare for next block =======================
                // increment next block number
                scope_context.next_block_number += 1;

                // if end of scope, remove the scope context
                if is_end_of_context {
                    request_scopes.remove(&endpoint_context_id);
                    break;
                }
                // cleanup if section is finished
                else if is_end_of_section {
                    // increment section index
                    scope_context.next_section_index += 1;
                    // close and remove the current section queue sender
                    if let Some(sender) =
                        scope_context.current_queue_sender.take()
                    {
                        sender.close_channel();
                    }
                }
                // ========================================================

                // check if next block is in cache for next iteration
                if let Some(next_cached_block) = scope_context
                    .cached_blocks
                    .remove(&scope_context.next_block_number)
                {
                    // check if block is end of section
                    is_end_of_section = next_cached_block
                        .block_header
                        .flags_and_timestamp
                        .is_end_of_section();
                    // check if block is end of scope
                    is_end_of_context = next_cached_block
                        .block_header
                        .flags_and_timestamp
                        .is_end_of_context();
                    // set next block
                    next_block = next_cached_block;

                    // update section index from next block
                    section_index = next_block.block_header.section_index;
                }
                // no more blocks in cache, break
                else {
                    break;
                }
            }

            new_blocks
        }
        // Case 3: if the block is not the next expected block in the current section,
        // put it into the block cache
        else {
            // check if block is already in cache
            // TODO #173: this should not happen, we should make sure duplicate blocks are dropped before
            if scope_context.cached_blocks.contains_key(&block_number) {
                log::warn!(
                    "Block {block_number} already in cache, dropping block"
                );
            }

            // add block to cache
            scope_context.cached_blocks.insert(block_number, block);

            vec![]
        }
    }

    pub fn get_new_context_id(&self) -> OutgoingContextId {
        *self.current_context_id.borrow_mut() += 1;
        *self.current_context_id.borrow()
    }

    /// Adds a new observer for incoming blocks with a specific scope id and block index
    /// Returns a receiver that can be awaited to get the incoming sections
    pub fn register_incoming_block_observer(
        &self,
        context_id: OutgoingContextId,
        section_index: OutgoingSectionIndex,
    ) -> UnboundedReceiver<IncomingSection> {
        let (tx, rx) = create_unbounded_channel::<IncomingSection>();
        let tx = Rc::new(RefCell::new(tx));

        // create observer callback for scope id + block index
        let observer = move |blocks: IncomingSection| {
            tx.clone().borrow_mut().start_send(blocks).unwrap();
        };

        // add new scope observer
        self.section_observers
            .borrow_mut()
            .insert((context_id, section_index), Box::new(observer));

        rx
    }

    /// Waits for incoming response block with a specific scope id and block index
    pub async fn wait_for_incoming_response_block(
        &self,
        context_id: OutgoingContextId,
        section_index: OutgoingSectionIndex,
    ) -> Option<IncomingSection> {
        let _rx =
            self.register_incoming_block_observer(context_id, section_index);
        // Await the result from the callback
        // FIXME #174
        None
        // rx.next().await
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use log::info;
    use ntest_timeout::timeout;
    use tokio::task::yield_now;
    use datex_macros::async_test;
    use crate::global::dxb_block::{DXBBlock, IncomingSection};
    use crate::global::protocol_structures::block_header::{BlockHeader, BlockType, FlagsAndTimestamp};
    use crate::global::protocol_structures::routing_header::RoutingHeader;
    use crate::network::com_interfaces::com_interface::properties::InterfaceDirection;
    use crate::values::core_values::endpoint::Endpoint;

    lazy_static::lazy_static! {
        pub static ref TEST_ENDPOINT_ORIGIN: Endpoint = Endpoint::from_str("@origin").unwrap();
        pub static ref TEST_ENDPOINT_A: Endpoint = Endpoint::from_str("@test-a").unwrap();
        pub static ref TEST_ENDPOINT_B: Endpoint = Endpoint::from_str("@test-b").unwrap();
    }

    // #[tokio::test]
    // async fn receive_single_block() {
    //     let (com_hub, interface_proxy, mut com_hub_sections_receiver) =
    //         get_default_mock_setup_with_com_hub().await;
    //
    //     let context_id = com_hub.block_handler.get_new_context_id();
    //
    //     // Create a single DXB block
    //     let mut block = DXBBlock {
    //         block_header: BlockHeader {
    //             context_id,
    //             flags_and_timestamp: FlagsAndTimestamp::new()
    //                 .with_is_end_of_section(true)
    //                 .with_is_end_of_context(true),
    //             ..BlockHeader::default()
    //         },
    //         routing_header: RoutingHeader::default()
    //             .with_sender(TEST_ENDPOINT_A.clone())
    //             .to_owned(),
    //         ..DXBBlock::default()
    //     };
    //     block.set_receivers(vec![TEST_ENDPOINT_ORIGIN.clone()]);
    //     let block_endpoint_context_id = block.get_endpoint_context_id();
    //
    //     // Send as incoming data into the interface
    //     let (_, mut interface_in_sender) =
    //         interface_proxy.create_and_init_socket(InterfaceDirection::InOut, 0);
    //     interface_in_sender.send(block.to_bytes()).await.unwrap();
    //
    //     // wait a tick to allow processing
    //     yield_now().await;
    //
    //     // block must be in incoming_sections_queue
    //     let block = get_next_received_single_block_from_receiver(
    //         &mut com_hub_sections_receiver,
    //     )
    //         .await;
    //
    //     assert_eq!(block.get_endpoint_context_id(), block_endpoint_context_id);
    // }
    //
    // #[async_test]
    // async fn receive_multiple_blocks() {
    //     let (com_hub, com_interface, mut com_hub_sections_receiver) =
    //         get_default_mock_setup_with_com_hub().await;
    //
    //     let context_id = com_hub.block_handler.get_new_context_id();
    //     let section_index = 42;
    //
    //     // Create a single DXB block
    //     let mut blocks = vec![
    //         DXBBlock {
    //             block_header: BlockHeader {
    //                 context_id,
    //                 section_index,
    //                 block_number: 0,
    //                 flags_and_timestamp: FlagsAndTimestamp::new()
    //                     .with_is_end_of_section(false)
    //                     .with_is_end_of_context(false),
    //                 ..BlockHeader::default()
    //             },
    //             routing_header: RoutingHeader::default()
    //                 .with_sender(TEST_ENDPOINT_A.clone())
    //                 .to_owned(),
    //             ..DXBBlock::default()
    //         },
    //         DXBBlock {
    //             block_header: BlockHeader {
    //                 context_id,
    //                 section_index,
    //                 block_number: 1,
    //                 flags_and_timestamp: FlagsAndTimestamp::new()
    //                     .with_is_end_of_section(true)
    //                     .with_is_end_of_context(true),
    //                 ..BlockHeader::default()
    //             },
    //             routing_header: RoutingHeader::default()
    //                 .with_sender(TEST_ENDPOINT_A.clone())
    //                 .to_owned(),
    //             ..DXBBlock::default()
    //         },
    //     ];
    //
    //     // Set receiver for each block
    //     for block in &mut blocks {
    //         block.set_receivers(vec![TEST_ENDPOINT_ORIGIN.clone()]);
    //     }
    //
    //     let (_, mut interface_in_sender) =
    //         com_interface.create_and_init_socket(InterfaceDirection::InOut, 0);
    //
    //     // 1. Send first block
    //     let block_bytes = blocks[0].to_bytes();
    //     interface_in_sender.send(block_bytes).await.unwrap();
    //
    //     // wait a tick to allow processing
    //     yield_now().await;
    //
    //     // block must be in incoming_sections_queue
    //     let mut section = com_hub_sections_receiver.next().await.unwrap();
    //     match &section {
    //         IncomingSection::BlockStream((
    //                                          Some(blocks),
    //                                          incoming_context_section_id,
    //                                      )) => {
    //             // section must match
    //             assert_eq!(
    //                 incoming_context_section_id.section_index,
    //                 section_index
    //             );
    //             // blocks queue must contain the first block
    //             assert!(section.next().await.is_some());
    //         }
    //         _ => core::panic!("Expected a BlockStream section"),
    //     }
    //
    //     // 2. Send second block
    //     let block_bytes = blocks[1].to_bytes();
    //     interface_in_sender.send(block_bytes).await.unwrap();
    //
    //     // wait a tick to allow processing
    //     yield_now().await;
    //
    //     // no new incoming sections, old section receives new blocks
    //     // block must be a block stream
    //     match &section {
    //         IncomingSection::BlockStream((
    //                                          Some(blocks),
    //                                          incoming_context_section_id,
    //                                      )) => {
    //             // section must match
    //             assert_eq!(
    //                 incoming_context_section_id.section_index,
    //                 section_index
    //             );
    //             // blocks queue length must be 2 (was not yet drained)
    //             assert_eq!(section.drain().await.len(), 1);
    //         }
    //         _ => core::panic!("Expected a BlockStream section"),
    //     }
    // }
    //
    // #[async_test]
    // async fn receive_multiple_blocks_wrong_order() {
    //     let (com_hub, com_interface, mut com_hub_sections_receiver) =
    //         get_default_mock_setup_with_com_hub().await;
    //
    //     let context_id = com_hub.block_handler.get_new_context_id();
    //     let section_index = 42;
    //
    //     // Create a single DXB block
    //     let mut blocks = vec![
    //         DXBBlock {
    //             block_header: BlockHeader {
    //                 context_id,
    //                 section_index,
    //                 block_number: 1,
    //                 flags_and_timestamp: FlagsAndTimestamp::new()
    //                     .with_is_end_of_section(true)
    //                     .with_is_end_of_context(true),
    //                 ..BlockHeader::default()
    //             },
    //             routing_header: RoutingHeader::default()
    //                 .with_sender(TEST_ENDPOINT_A.clone())
    //                 .to_owned(),
    //             ..DXBBlock::default()
    //         },
    //         DXBBlock {
    //             block_header: BlockHeader {
    //                 context_id,
    //                 section_index,
    //                 block_number: 0,
    //                 flags_and_timestamp: FlagsAndTimestamp::new()
    //                     .with_is_end_of_section(false)
    //                     .with_is_end_of_context(false),
    //                 ..BlockHeader::default()
    //             },
    //             routing_header: RoutingHeader::default()
    //                 .with_sender(TEST_ENDPOINT_A.clone())
    //                 .to_owned(),
    //             ..DXBBlock::default()
    //         },
    //     ];
    //
    //     // Set receiver for each block
    //     for block in &mut blocks {
    //         block.set_receivers(vec![TEST_ENDPOINT_ORIGIN.clone()]);
    //     }
    //
    //     let (_, mut interface_in_sender) =
    //         com_interface.create_and_init_socket(InterfaceDirection::InOut, 0);
    //
    //     // 1. Send first block
    //     let block_bytes = blocks[0].to_bytes();
    //     interface_in_sender.send(block_bytes).await.unwrap();
    //
    //     yield_now().await;
    //
    //     // 2. Send second block
    //     let block_bytes = blocks[1].to_bytes();
    //     interface_in_sender.send(block_bytes).await.unwrap();
    //
    //     yield_now().await;
    //
    //     // block must be in incoming_sections_queue
    //     let mut section = com_hub_sections_receiver.next().await.unwrap();
    //     // block must be a block stream
    //     match &section {
    //         IncomingSection::BlockStream((
    //                                          Some(blocks),
    //                                          incoming_context_section_id,
    //                                      )) => {
    //             // section must match
    //             assert_eq!(
    //                 incoming_context_section_id.section_index.clone(),
    //                 section_index
    //             );
    //             // blocks queue length must be 2
    //             let blocks = section.drain().await;
    //             assert_eq!(blocks.len(), 2);
    //
    //             // check order:
    //             // first block must have block number 0
    //             let block = blocks.first().unwrap();
    //             assert_eq!(block.block_header.block_number, 0);
    //             // second block must have block number 1
    //             let block = blocks.get(1).unwrap();
    //             assert_eq!(block.block_header.block_number, 1);
    //         }
    //         _ => core::panic!("Expected a BlockStream section"),
    //     }
    // }
    //
    // #[async_test]
    // async fn receive_multiple_sections() {
    //     let (com_hub, com_interface, mut com_hub_sections_receiver) =
    //         get_default_mock_setup_with_com_hub().await;
    //
    //     let context_id = com_hub.block_handler.get_new_context_id();
    //     let section_index_1 = 42;
    //     let section_index_2 = 43;
    //
    //     // Create a single DXB block
    //     let mut blocks = vec![
    //         // first section
    //         DXBBlock {
    //             block_header: BlockHeader {
    //                 context_id,
    //                 section_index: section_index_1,
    //                 block_number: 0,
    //                 flags_and_timestamp: FlagsAndTimestamp::new()
    //                     .with_is_end_of_section(false)
    //                     .with_is_end_of_context(false),
    //                 ..BlockHeader::default()
    //             },
    //             routing_header: RoutingHeader::default()
    //                 .with_sender(TEST_ENDPOINT_A.clone())
    //                 .to_owned(),
    //             ..DXBBlock::default()
    //         },
    //         DXBBlock {
    //             block_header: BlockHeader {
    //                 context_id,
    //                 section_index: section_index_1,
    //                 block_number: 1,
    //                 flags_and_timestamp: FlagsAndTimestamp::new()
    //                     .with_is_end_of_section(true)
    //                     .with_is_end_of_context(false),
    //                 ..BlockHeader::default()
    //             },
    //             routing_header: RoutingHeader::default()
    //                 .with_sender(TEST_ENDPOINT_A.clone())
    //                 .to_owned(),
    //             ..DXBBlock::default()
    //         },
    //         // second section, end of context
    //         DXBBlock {
    //             block_header: BlockHeader {
    //                 context_id,
    //                 section_index: section_index_2,
    //                 block_number: 2,
    //                 flags_and_timestamp: FlagsAndTimestamp::new()
    //                     .with_is_end_of_section(false)
    //                     .with_is_end_of_context(false),
    //                 ..BlockHeader::default()
    //             },
    //             routing_header: RoutingHeader::default()
    //                 .with_sender(TEST_ENDPOINT_A.clone())
    //                 .to_owned(),
    //             ..DXBBlock::default()
    //         },
    //         DXBBlock {
    //             block_header: BlockHeader {
    //                 context_id,
    //                 section_index: section_index_2,
    //                 block_number: 3,
    //                 flags_and_timestamp: FlagsAndTimestamp::new()
    //                     .with_is_end_of_section(true)
    //                     .with_is_end_of_context(true),
    //                 ..BlockHeader::default()
    //             },
    //             routing_header: RoutingHeader::default()
    //                 .with_sender(TEST_ENDPOINT_A.clone())
    //                 .to_owned(),
    //             ..DXBBlock::default()
    //         },
    //     ];
    //
    //     // Set receiver for each block
    //     for block in &mut blocks {
    //         block.set_receivers(vec![TEST_ENDPOINT_ORIGIN.clone()]);
    //     }
    //
    //     let (_, mut interface_in_sender) =
    //         com_interface.create_and_init_socket(InterfaceDirection::InOut, 0);
    //
    //     // 1. Send first block
    //     let block_bytes = blocks[0].to_bytes();
    //     interface_in_sender.send(block_bytes).await.unwrap();
    //
    //     yield_now().await;
    //
    //     // block must be in incoming_sections_queue
    //     let mut section = com_hub_sections_receiver.next().await.unwrap();
    //     // block must be a block stream
    //     match &section {
    //         IncomingSection::BlockStream((
    //                                          Some(blocks),
    //                                          incoming_context_section_id,
    //                                      )) => {
    //             // section must match
    //             assert_eq!(
    //                 incoming_context_section_id.section_index,
    //                 section_index_1
    //             );
    //             // block queue must contain the first block
    //             assert!(section.next().await.is_some());
    //         }
    //         _ => core::panic!("Expected a BlockStream section"),
    //     }
    //
    //     // 2. Send second block
    //     let block_bytes = blocks[1].to_bytes();
    //     interface_in_sender.send(block_bytes).await.unwrap();
    //
    //     yield_now().await;
    //
    //     // block must be a block stream
    //     match &section {
    //         IncomingSection::BlockStream((
    //                                          Some(blocks),
    //                                          incoming_context_section_id,
    //                                      )) => {
    //             // section must match
    //             assert_eq!(
    //                 incoming_context_section_id.section_index,
    //                 section_index_1
    //             );
    //
    //             // blocks queue length must be 1
    //             assert_eq!(section.drain().await.len(), 1);
    //         }
    //         _ => core::panic!("Expected a BlockStream section"),
    //     }
    //
    //     // 3. Send third block
    //     let block_bytes = blocks[2].to_bytes();
    //     interface_in_sender.send(block_bytes).await.unwrap();
    //
    //     yield_now().await;
    //
    //     // block must be in incoming_sections_queue
    //     let mut section = com_hub_sections_receiver.next().await.unwrap();
    //     // block must be a block stream
    //     match &section {
    //         IncomingSection::BlockStream((
    //                                          Some(blocks),
    //                                          incoming_context_section_id,
    //                                      )) => {
    //             // section must match
    //             assert_eq!(
    //                 incoming_context_section_id.section_index,
    //                 section_index_2
    //             );
    //             // block queue must contain the first block
    //             assert!(section.next().await.is_some());
    //         }
    //         _ => core::panic!("Expected a BlockStream section"),
    //     }
    //
    //     // 4. Send fourth block
    //     let block_bytes = blocks[3].to_bytes();
    //     interface_in_sender.send(block_bytes).await.unwrap();
    //
    //     yield_now().await;
    //
    //     // block must not be in incoming_sections_queue
    //     // block must be a block stream
    //     match &section {
    //         IncomingSection::BlockStream((
    //                                          Some(blocks),
    //                                          incoming_context_section_id,
    //                                      )) => {
    //             // section must match
    //             assert_eq!(
    //                 incoming_context_section_id.section_index,
    //                 section_index_2
    //             );
    //             // blocks queue length must be 1
    //             assert_eq!(section.drain().await.len(), 1);
    //         }
    //         _ => core::panic!("Expected a BlockStream section"),
    //     }
    // }
    //
    // #[async_test]
    // #[timeout(2000)]
    // async fn await_response_block() {
    //     let (com_hub, com_interface, mut com_hub_sections_receiver) =
    //         get_default_mock_setup_with_com_hub().await;
    //
    //     let context_id = com_hub.block_handler.get_new_context_id();
    //     let section_index = 42;
    //
    //     // Create a single DXB block
    //     let mut block = DXBBlock {
    //         block_header: BlockHeader {
    //             context_id,
    //             section_index,
    //             flags_and_timestamp: FlagsAndTimestamp::new()
    //                 .with_block_type(BlockType::Response)
    //                 .with_is_end_of_section(true)
    //                 .with_is_end_of_context(true),
    //             ..BlockHeader::default()
    //         },
    //         routing_header: RoutingHeader::default()
    //             .with_sender(TEST_ENDPOINT_A.clone())
    //             .to_owned(),
    //         ..DXBBlock::default()
    //     };
    //     block.set_receivers(vec![TEST_ENDPOINT_ORIGIN.clone()]);
    //
    //     let (_, mut interface_in_sender) =
    //         com_interface.create_and_init_socket(InterfaceDirection::InOut, 0);
    //
    //     // set observer for the block
    //     let mut rx = com_hub
    //         .block_handler
    //         .register_incoming_block_observer(context_id, section_index);
    //
    //     // Put into incoming queue of mock interface
    //     let block_bytes = block.to_bytes();
    //     interface_in_sender.send(block_bytes).await.unwrap();
    //
    //     yield_now().await;
    //
    //     // await receiver
    //     let response = rx.next().await.unwrap();
    //
    //     // IncomingSection must be a SingleBlock
    //     match response {
    //         IncomingSection::SingleBlock((Some(block), _)) => {
    //             info!("section: {block:?}");
    //             assert_eq!(block.block_header.context_id, context_id);
    //             assert_eq!(block.block_header.section_index, section_index);
    //         }
    //         _ => core::panic!("Expected a SingleBlock section"),
    //     }
    // }
}