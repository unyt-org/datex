use crate::network::helpers::{
    mock_setup::{
        CollectedBlockType, MockupSetupData, TEST_ENDPOINT_A, TEST_ENDPOINT_B,
        TEST_ENDPOINT_ORIGIN, get_collected_outgoing_blocks_from_receiver,
        get_collected_received_blocks_from_receiver,
        get_default_mock_setup_with_com_hub,
        get_default_mock_setup_with_two_connected_com_hubs,
        get_mock_setup_with_com_hub,
        get_next_outgoing_block_from_receiver,
        get_next_received_single_block_from_receiver, send_block_with_body,
        send_empty_block, send_multiple_blocks_to_local,
    },
    mockup_interface::MockupInterfaceSetupData,
};
use datex_core::{
    global::{
        dxb_block::{DXBBlock},
        protocol_structures::{
            block_header::{BlockHeader, FlagsAndTimestamp},
            encrypted_header::{self, EncryptedHeader},
            routing_header::{RoutingHeader, SignatureType},
        },
    },
    network::{
        com_hub::{
            ComHub, InterfacePriority,
            metadata::{ComHubMetadata},
        },
        com_interfaces::com_interface::{
            ComInterface, ComInterfaceProxy,
            implementation::ComInterfaceSyncFactory,
            properties::{InterfaceDirection, InterfaceProperties},
            state::ComInterfaceState,
        },
    },
    runtime::AsyncContext,
    serde::serializer::to_value_container,
    stdlib::rc::Rc,
    task::{create_unbounded_channel, sleep},
    values::core_values::endpoint::Endpoint,
};
use datex_macros::async_test;
use tokio::task::yield_now;

/// Creates a mock ComHub for testing without a connected channel
fn create_mock_com_hub() -> Rc<ComHub> {
    let (sender, receiver) = create_unbounded_channel();
    ComHub::create(Endpoint::default(), sender, AsyncContext::new())
}

#[async_test]
pub async fn test_add_and_remove() {
    let com_hub = create_mock_com_hub();
    let mockup_interface_with_receivers =
        ComInterface::create_sync_from_setup_data(
            MockupInterfaceSetupData::new("test"),
            AsyncContext::default()
        )
        .unwrap();

    let uuid = mockup_interface_with_receivers.0.uuid().clone();

    com_hub
        .register_com_interface(
            mockup_interface_with_receivers,
            InterfacePriority::default(),
        )
        .unwrap();

    assert!(com_hub.remove_interface(uuid).is_ok());
}

#[async_test]
pub async fn test_multiple_add() {
    let com_hub = create_mock_com_hub();

    let mockup_interface1 = ComInterface::create_sync_from_setup_data(
        MockupInterfaceSetupData::new("mockup_interface1"),
        AsyncContext::default()
    )
    .unwrap();
    let mockup_interface2 = ComInterface::create_sync_from_setup_data(
        MockupInterfaceSetupData::new("mockup_interface2"),
        AsyncContext::default()
    )
    .unwrap();

    com_hub
        .register_com_interface(mockup_interface1, InterfacePriority::default())
        .unwrap();
    com_hub
        .register_com_interface(mockup_interface2, InterfacePriority::default())
        .unwrap();
}

fn metadata_sockets(
    com_hub_metadata: ComHubMetadata,
) -> Vec<(Option<Endpoint>, Option<i8>)> {
    com_hub_metadata
        .interfaces
        .into_iter()
        .flat_map(|e| {
            e.sockets
                .into_iter()
                .map(|s| (s.endpoint, s.properties.map(|p| p.distance)))
        })
        .collect::<Vec<_>>()
}

#[async_test]
#[timeout(1000)]
async fn create_hello_connection() {
    let ((com_hub_mut_a, ..), (com_hub_mut_b, ..)) =
        get_default_mock_setup_with_two_connected_com_hubs().await;

    yield_now().await;

    let com_hub_a_sockets = metadata_sockets(com_hub_mut_a.metadata());
    assert!(
        com_hub_a_sockets.contains(&(Some(TEST_ENDPOINT_B.clone()), Some(1)))
    );
    let com_hub_b_sockets = metadata_sockets(com_hub_mut_b.metadata());
    assert!(
        com_hub_b_sockets.contains(&(Some(Endpoint::LOCAL.clone()), Some(0)))
    );

    let com_hub_b_sockets = metadata_sockets(com_hub_mut_b.metadata());
    assert!(
        com_hub_b_sockets.contains(&(Some(TEST_ENDPOINT_A.clone()), Some(1)))
    );
    assert!(
        com_hub_b_sockets.contains(&(Some(Endpoint::LOCAL.clone()), Some(0)))
    );
}

#[async_test]
pub async fn test_send() {
    let (com_hub, mut interface_proxy, _) =
        get_default_mock_setup_with_com_hub().await;

    // set up socket that goes to TEST_ENDPOINT_B (direct connection)
    interface_proxy.create_and_init_socket_with_direct_endpoint(
        InterfaceDirection::Out,
        1,
        TEST_ENDPOINT_B.clone(),
    );

    yield_now().await;

    // send block via com hub to proxy interface
    let sent_block = send_block_with_body(
        std::slice::from_ref(&TEST_ENDPOINT_B),
        b"Hello world!",
        &com_hub,
    )
    .await;

    // hello block, skip
    interface_proxy.event_receiver.next().await.unwrap();
    // get next block that was sent
    let recorded_block = get_next_outgoing_block_from_receiver(
        &mut interface_proxy.event_receiver,
    )
    .await
    .0;
    assert_eq!(recorded_block.body, sent_block.body);
}

#[async_test]
pub async fn send_block_to_invalid_receiver() {
    // init without fallback interfaces
    let (com_hub, interface_proxy, ..) =
        get_mock_setup_with_com_hub(MockupSetupData {
            interface_properties: Default::default(),
            interface_priority: InterfacePriority::None,
            ..Default::default()
        })
        .await;

    assert!(
        send_empty_block(std::slice::from_ref(&TEST_ENDPOINT_B), &com_hub).await.is_err()
    );
}

#[async_test]
pub async fn send_block_to_multiple_endpoints() {
    let (com_hub, mut interface_proxy, _) =
        get_default_mock_setup_with_com_hub().await;

    let (socket_uuid, _) =
        interface_proxy.create_and_init_socket(InterfaceDirection::InOut, 0);
    yield_now().await;

    com_hub
        .socket_manager()
        .borrow_mut()
        .register_socket_endpoint(
            socket_uuid.clone(),
            TEST_ENDPOINT_A.clone(),
            1,
        )
        .unwrap();

    com_hub
        .socket_manager()
        .borrow_mut()
        .register_socket_endpoint(
            socket_uuid.clone(),
            TEST_ENDPOINT_B.clone(),
            1,
        )
        .unwrap();

    // send block to multiple receivers
    let sent_block = send_block_with_body(
        &[TEST_ENDPOINT_A.clone(), TEST_ENDPOINT_B.clone()],
        b"Hello world",
        &com_hub,
    )
    .await;

    // hello block, skip
    interface_proxy.event_receiver.next().await.unwrap();
    // get next block that was sent
    let recorded_block = get_next_outgoing_block_from_receiver(
        &mut interface_proxy.event_receiver,
    )
    .await
    .0;
    assert_eq!(recorded_block.body, sent_block.body);
}

#[async_test]
pub async fn send_blocks_to_multiple_endpoint_sockets() {
    let (com_hub, mut interface_proxy, _) =
        get_default_mock_setup_with_com_hub().await;

    // create two separate sockets for each endpoint

    let (socket_uuid_a, _) = interface_proxy
        .create_and_init_socket_with_direct_endpoint(
            InterfaceDirection::InOut,
            1,
            TEST_ENDPOINT_A.clone(),
        );
    yield_now().await;

    let (socket_uuid_b, _) = interface_proxy
        .create_and_init_socket_with_direct_endpoint(
            InterfaceDirection::InOut,
            1,
            TEST_ENDPOINT_B.clone(),
        );
    yield_now().await;

    // send block to multiple receivers
    assert!(
        send_empty_block(
            &[TEST_ENDPOINT_A.clone(), TEST_ENDPOINT_B.clone()],
            &com_hub,
        )
        .await.is_ok()
    );

    let blocks = get_collected_outgoing_blocks_from_receiver(
        &mut interface_proxy.event_receiver,
        2,
    )
    .await;
    let block_uuids = blocks
        .into_iter()
        .map(|(_, socket_uuid)| socket_uuid)
        .collect::<Vec<_>>();

    assert!(block_uuids.contains(&socket_uuid_a));
    assert!(block_uuids.contains(&socket_uuid_b));
}


#[async_test]
pub async fn test_receive() {
    let (_, interface_proxy, mut incoming_sections_receiver) = get_default_mock_setup_with_com_hub().await;

    // receive block
    let mut block = DXBBlock {
        body: vec![0x01, 0x02, 0x03],
        encrypted_header: EncryptedHeader {
            flags: encrypted_header::Flags::new()
                .with_user_agent(encrypted_header::UserAgent::Unused11),
            ..Default::default()
        },
        ..DXBBlock::default()
    };
    block.set_receivers(vec![TEST_ENDPOINT_ORIGIN.clone()]);
    block.recalculate_struct();

    let (_, mut incoming_blocks_sender) =
        interface_proxy.create_and_init_socket(InterfaceDirection::In, 1);
    yield_now().await;

    let block_bytes = block.to_bytes().unwrap();
    incoming_blocks_sender
        .start_send(block_bytes.as_slice().to_vec())
        .unwrap();

    yield_now().await;

    let last_block = get_next_received_single_block_from_receiver(
        &mut incoming_sections_receiver,
    )
    .await;
    assert_eq!(last_block.raw_bytes.clone().unwrap(), block_bytes);
}

#[async_test]
pub async fn unencrypted_signature_prepare_block_com_hub() {
    let (com_hub, interface_proxy, mut incoming_sections_receiver) =
        get_default_mock_setup_with_com_hub().await;

    // receive block
    let mut block = DXBBlock {
        body: vec![0x01, 0x02, 0x03],
        encrypted_header: EncryptedHeader {
            flags: encrypted_header::Flags::new()
                .with_user_agent(encrypted_header::UserAgent::Unused11),
            ..Default::default()
        },
        ..DXBBlock::default()
    };
    block.set_receivers(vec![TEST_ENDPOINT_ORIGIN.clone()]);
    block.recalculate_struct();
    block
        .routing_header
        .flags
        .set_signature_type(SignatureType::Unencrypted);
    block = com_hub.prepare_own_block(block).await.unwrap();
    let block_bytes = block.to_bytes().unwrap();

    let (_, mut incoming_blocks_sender) =
        interface_proxy.create_and_init_socket(InterfaceDirection::In, 0);
    yield_now().await;

    incoming_blocks_sender
        .start_send(block_bytes.as_slice().to_vec())
        .unwrap();

    yield_now().await;

    let last_block = get_next_received_single_block_from_receiver(
        &mut incoming_sections_receiver,
    )
    .await;
    assert_eq!(last_block.raw_bytes.clone().unwrap(), block_bytes);
    assert_eq!(block.signature, last_block.signature);

    assert!(com_hub.validate_block(&last_block).await.unwrap());
}

#[async_test]
pub async fn encrypted_signature_prepare_block_com_hub() {
    let (com_hub, interface_proxy, mut incoming_sections_receiver) =
        get_default_mock_setup_with_com_hub().await;

    // receive block
    let mut block = DXBBlock {
        body: vec![0x01, 0x02, 0x03],
        encrypted_header: EncryptedHeader {
            flags: encrypted_header::Flags::new()
                .with_user_agent(encrypted_header::UserAgent::Unused11),
            ..Default::default()
        },
        ..DXBBlock::default()
    };

    block.set_receivers(vec![TEST_ENDPOINT_ORIGIN.clone()]);
    block.recalculate_struct();

    block
        .routing_header
        .flags
        .set_signature_type(SignatureType::Encrypted);
    block = com_hub.prepare_own_block(block).await.unwrap();
    let block_bytes = block.to_bytes().unwrap();

    let (_, mut incoming_blocks_sender) =
        interface_proxy.create_and_init_socket(InterfaceDirection::In, 0);
    yield_now().await;

    incoming_blocks_sender
        .start_send(block_bytes.as_slice().to_vec())
        .unwrap();

    yield_now().await;

    let last_block = get_next_received_single_block_from_receiver(
        &mut incoming_sections_receiver,
    )
    .await;
    assert_eq!(last_block.raw_bytes.clone().unwrap(), block_bytes);
    assert_eq!(block.signature, last_block.signature);

    assert!(com_hub.validate_block(&last_block).await.unwrap());
}

#[async_test]
pub async fn test_receive_multiple_blocks_single_section() {
    let (_, interface_proxy, mut incoming_sections_receiver) = get_default_mock_setup_with_com_hub().await;


    let mut blocks = vec![
        DXBBlock {
            routing_header: RoutingHeader::default(),
            block_header: BlockHeader {
                section_index: 0,
                block_number: 0,
                flags_and_timestamp: FlagsAndTimestamp::new()
                    .with_is_end_of_section(false)
                    .with_is_end_of_context(false),
                ..Default::default()
            },
            ..Default::default()
        },
        DXBBlock {
            routing_header: RoutingHeader::default(),
            block_header: BlockHeader {
                section_index: 0,
                block_number: 1,
                flags_and_timestamp: FlagsAndTimestamp::new()
                    .with_is_end_of_section(false)
                    .with_is_end_of_context(false),
                ..Default::default()
            },
            ..Default::default()
        },
        DXBBlock {
            routing_header: RoutingHeader::default(),
            block_header: BlockHeader {
                section_index: 0,
                block_number: 2,
                flags_and_timestamp: FlagsAndTimestamp::new()
                    .with_is_end_of_section(true)
                    .with_is_end_of_context(true),
                ..Default::default()
            },
            ..Default::default()
        },
    ];
    let blocks_count = blocks.len();

    let (_, mut incoming_blocks_sender) =
        interface_proxy.create_and_init_socket(InterfaceDirection::In, 0);
    yield_now().await;

    // send blocks via incoming_blocks_sender
    send_multiple_blocks_to_local(&mut incoming_blocks_sender, &mut blocks)
        .await;

    // collect received blocks from incoming_sections_receiver
    let incoming_blocks = get_collected_received_blocks_from_receiver(
        &mut incoming_sections_receiver,
        CollectedBlockType::BlockStream,
        blocks_count,
    )
    .await;

    for (incoming_block, block) in incoming_blocks.iter().zip(blocks.iter()) {
        assert_eq!(
            incoming_block.raw_bytes.clone().unwrap(),
            block.to_bytes().unwrap()
        );
    }
}

#[async_test]
pub async fn test_receive_multiple_separate_blocks() {
    let (_, interface_proxy, mut incoming_sections_receiver) = get_default_mock_setup_with_com_hub().await;


    let mut blocks = vec![
        DXBBlock {
            routing_header: RoutingHeader::default(),
            block_header: BlockHeader {
                section_index: 1,
                block_number: 0,
                ..Default::default()
            },
            ..Default::default()
        },
        DXBBlock {
            routing_header: RoutingHeader::default(),
            block_header: BlockHeader {
                section_index: 2,
                block_number: 0,
                ..Default::default()
            },
            ..Default::default()
        },
        DXBBlock {
            routing_header: RoutingHeader::default(),
            block_header: BlockHeader {
                section_index: 3,
                block_number: 0,
                ..Default::default()
            },
            ..Default::default()
        },
    ];
    let blocks_count = blocks.len();

    let (_, mut incoming_blocks_sender) =
        interface_proxy.create_and_init_socket(InterfaceDirection::In, 0);
    yield_now().await;

    // send blocks via incoming_blocks_sender
    send_multiple_blocks_to_local(&mut incoming_blocks_sender, &mut blocks)
        .await;

    // collect received blocks from incoming_sections_receiver
    let incoming_blocks = get_collected_received_blocks_from_receiver(
        &mut incoming_sections_receiver,
        CollectedBlockType::SingleBocks,
        blocks_count,
    )
    .await;

    for (incoming_block, block) in incoming_blocks.iter().zip(blocks.iter()) {
        assert_eq!(
            incoming_block.raw_bytes.clone().unwrap(),
            block.to_bytes().unwrap()
        );
    }
}

#[async_test]
pub async fn test_add_and_remove_interface_and_sockets() {
    let (com_hub, interface_proxy, _) =
        get_default_mock_setup_with_com_hub().await;

    // initial state with loopback interface and mockup interface
    {
        let interface_manager = com_hub.interface_manager();
        let socket_manager = com_hub.socket_manager();
        assert_eq!(interface_manager.borrow().interfaces.len(), 2); // loopback + mockup interface
        assert_eq!(socket_manager.borrow().sockets.len(), 1); // loopback
        assert_eq!(socket_manager.borrow().endpoint_sockets.len(), 1);
    }

    assert_eq!(
        interface_proxy.state.lock().unwrap().get(),
        ComInterfaceState::Connected
    );

    // add new socket without direct endpoint
    let (socket_uuid, _) =
        interface_proxy.create_and_init_socket(InterfaceDirection::InOut, 1);
    yield_now().await;

    {
        let socket_manager = com_hub.socket_manager();
        assert!(socket_manager.borrow().has_socket(&socket_uuid));
        assert_eq!(socket_manager.borrow().sockets.len(), 2);
        assert_eq!(socket_manager.borrow().endpoint_sockets.len(), 1);
    }

    // add new socket with direct endpoint
    let (socket_uuid, _) =
        interface_proxy.create_and_init_socket_with_direct_endpoint(InterfaceDirection::InOut, 1, TEST_ENDPOINT_A.clone());
    yield_now().await;

    {
        let socket_manager = com_hub.socket_manager();
        assert!(socket_manager.borrow().has_socket(&socket_uuid));
        assert_eq!(socket_manager.borrow().sockets.len(), 3);
        assert_eq!(socket_manager.borrow().endpoint_sockets.len(), 2);
    }

    let interface_uuid = interface_proxy.uuid.clone();

    // remove interface
    assert!(com_hub.remove_interface(interface_uuid).is_ok());

    {
        let interface_manager = com_hub.interface_manager();
        let socket_manager = com_hub.socket_manager();
        assert_eq!(interface_manager.borrow().interfaces.len(), 1); // loopback interface
        assert_eq!(socket_manager.borrow().sockets.len(), 1); // loopback socket
        assert_eq!(socket_manager.borrow().endpoint_sockets.len(), 1);
    }

    assert_eq!(
        interface_proxy.state.lock().unwrap().get(),
        ComInterfaceState::Destroyed
    );

    let socket_manager = com_hub.socket_manager();
    assert!(!socket_manager.borrow().has_socket(&socket_uuid))
}

#[async_test]
pub async fn test_basic_routing() {
    let ((com_hub_mut_a, ..), (_, mut incoming_sections_receiver_b, _)) =
        get_default_mock_setup_with_two_connected_com_hubs().await;

    yield_now().await;
    yield_now().await;

    let block_a_to_b = send_block_with_body(
        std::slice::from_ref(&TEST_ENDPOINT_B),
        b"Hello world",
        &com_hub_mut_a,
    )
    .await;

    yield_now().await;

    let last_block = get_next_received_single_block_from_receiver(
        &mut incoming_sections_receiver_b,
    )
    .await;
    assert_eq!(block_a_to_b.body, last_block.body);
}

#[async_test]
pub async fn register_factory() {
    let com_hub = create_mock_com_hub();
    com_hub.register_sync_interface_factory::<MockupInterfaceSetupData>();

    assert_eq!(
        com_hub
            .interface_manager()
            .borrow()
            .interface_factories
            .len(),
        1
    );
    assert!(
        com_hub
            .interface_manager()
            .borrow()
            .interface_factories
            .contains_key("mockup")
    );

    // create a new mockup interface from the com_hub
    let interface_uuid = com_hub
        .create_interface(
            "mockup",
            to_value_container(&MockupInterfaceSetupData::new("mockup"))
                .unwrap(),
            InterfacePriority::default(),
            AsyncContext::default()
        )
        .await
        .unwrap();

    assert_eq!(
        com_hub
            .interface_manager()
            .borrow()
            .get_interface_by_uuid(&interface_uuid)
            .properties()
            .interface_type,
        "mockup"
    );
}

#[async_test]
pub async fn test_reconnect() {
    let com_hub = create_mock_com_hub();

    // TODO: refactor using proxy

    // create a new interface, open it and add it to the com_hub
    let (base_interface, interface_with_receivers) =
        ComInterfaceProxy::create_interface(InterfaceProperties::default(), AsyncContext::default());

    // add base_interface to com_hub
    com_hub
        .register_com_interface(
            interface_with_receivers,
            InterfacePriority::default(),
        )
        .unwrap();

    // check that the interface is connected
    assert_eq!(
        base_interface.state.lock().unwrap().get(),
        ComInterfaceState::Connected
    );

    // check that the interface is in the com_hub
    assert_eq!(com_hub.interface_manager().borrow().interfaces.len(), 2); // loopback + base_interface
    assert!(com_hub.has_interface(&base_interface.uuid));

    // simulate a disconnection by closing the interface
    // This action is normally done by the interface itself
    // but we do it manually here to test the reconnection
    // TODO: reconnect
    // // check that the interface is not connected
    // // and that the close_timestamp is set
    // assert_eq!(
    //     base_interface.state.lock().unwrap().get(),
    //     ComInterfaceState::NotConnected
    // );
    //
    // assert!(
    //     base_interface
    //         .com_interface
    //         .properties()
    //         .close_timestamp
    //         .is_some()
    // );
    //
    // // the interface should not be reconnected yet
    // yield_now().await;
    //
    // assert_eq!(
    //     base_interface.com_interface.current_state(),
    //     ComInterfaceState::NotConnected
    // );
    //
    // // wait for the reconnection to happen
    // tokio::time::sleep(Duration::from_secs(1)).await;
    //
    // // check that the interface is connected again
    // // and that the close_timestamp is reset
    // yield_now().await;
    //
    // assert_eq!(
    //     base_interface.com_interface.current_state(),
    //     ComInterfaceState::Connected
    // );
}
