use crate::network::helpers::{
    mock_setup::{
        TEST_ENDPOINT_A, TEST_ENDPOINT_B, TEST_ENDPOINT_ORIGIN,
        get_collected_received_blocks_from_receiver,
        get_mock_setup_with_com_hub,
        send_block_with_body, send_empty_block,
    },
    mockup_interface::{MockupInterfaceSetupData},
};
use datex_core::{
    global::{
        dxb_block::DXBBlock,
        protocol_structures::{
            block_header::BlockHeader,
            encrypted_header::{self, EncryptedHeader},
            routing_header::{RoutingHeader, SignatureType},
        },
    },
    network::{
        com_hub::{ComHub, InterfacePriority},
        com_interfaces::{
            com_interface::{
                ComInterface,
                properties::{InterfaceProperties, ReconnectionConfig},
                socket::SocketState,
                state::ComInterfaceState,
            },
            default_com_interfaces::base_interface::{
                BaseInterface, BaseInterfaceSetupData,
            },
        },
    },
    runtime::AsyncContext,
    serde::serializer::to_value_container,
    stdlib::{cell::RefCell, rc::Rc},
    values::core_values::endpoint::Endpoint,
};
use datex_macros::async_test;
use std::time::Duration;
use tokio::task::yield_now;
use datex_core::global::protocol_structures::block_header::FlagsAndTimestamp;
use datex_core::network::com_interfaces::com_interface::ComInterfaceEvent;
use datex_core::network::com_interfaces::com_interface::properties::InterfaceDirection;
use datex_core::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;
use datex_core::task::{create_unbounded_channel, sleep};
use crate::network::helpers::mock_setup::{get_default_mock_setup_with_com_hub, get_default_mock_setup_with_two_connected_com_hubs, get_next_received_single_block_from_receiver, get_mock_setup_with_interface, MockupSetupData, CollectedBlockType, send_multiple_blocks_to_local};

/// Creates a mock ComHub for testing without a connected channel
fn create_mock_com_hub() -> Rc<ComHub> {
    let (sender, receiver) = create_unbounded_channel();
    ComHub::create(
        Endpoint::default(),
        sender,
        AsyncContext::new(),
    )
}

#[async_test]
pub async fn test_add_and_remove() {

    let com_hub = create_mock_com_hub();
    let mockup_interface_with_receivers = ComInterface::create_sync_from_setup_data(MockupInterfaceSetupData::new("test"))
        .unwrap();

    let uuid = mockup_interface_with_receivers.0.uuid().clone();

    com_hub.register_com_interface(
        mockup_interface_with_receivers,
        InterfacePriority::default(),
    ).unwrap();

    assert!(com_hub.remove_interface(uuid).is_ok());
}

#[async_test]
pub async fn test_multiple_add() {
    let com_hub = create_mock_com_hub();

    let mockup_interface1 =
        ComInterface::create_sync_from_setup_data(
            MockupInterfaceSetupData::new("mockup_interface1"),
        )
        .unwrap();
    let mockup_interface2 =
        ComInterface::create_sync_from_setup_data(
            MockupInterfaceSetupData::new("mockup_interface2"),
        )
        .unwrap();

    com_hub.register_com_interface(
        mockup_interface1,
        InterfacePriority::default(),
    ).unwrap();
    com_hub.register_com_interface(
        mockup_interface2,
        InterfacePriority::default(),
    ).unwrap();

    // TODO: change test here, adding the same interface twice is not possible since it
    // doesnt implement clone - still it could be possible to add two interfaces with the same uuid
    assert!(
        com_hub
            .register_com_interface(
                mockup_interface1,
                InterfacePriority::default()
            )
            .is_err()
    );
    assert!(
        com_hub
            .register_com_interface(
                mockup_interface2,
                InterfacePriority::default()
            )
            .is_err()
    );
}

#[async_test]
pub async fn test_send() {
    let (com_hub, mut interface_proxy, _) = get_default_mock_setup_with_com_hub().await;

    // set up socket that goes to TEST_ENDPOINT_B
    let (socket_uuid, _) = interface_proxy.create_and_init_socket(
        InterfaceDirection::Out, 0
    );
    interface_proxy.register_socket_endpoint(
        socket_uuid,
        TEST_ENDPOINT_B.clone(),
        1,
    );

    // send block via com hub to proxy interface
    let sent_block = send_block_with_body(
        std::slice::from_ref(&TEST_ENDPOINT_B),
        b"Hello world!",
        &com_hub,
    )
    .await;

    // get next block that was sent
    let next_event = interface_proxy.event_receiver.next().await.unwrap();
    match next_event {
        ComInterfaceEvent::SendBlock(block, _) => {
            let block_bytes =
                DXBBlock::from_bytes(&block)
                    .await
                    .unwrap();
            assert_eq!(block_bytes.body, sent_block.body);
        }
        _ => panic!("Expected SendBlock event"),
    }
}

#[async_test]
pub async fn test_send_invalid_recipient() {
    // init without fallback interfaces
    let (com_hub, interface_proxy, ..) = get_mock_setup_with_com_hub(MockupSetupData {
        interface_properties: Default::default(),
        interface_priority: InterfacePriority::None,
        ..Default::default()
    }).await;

    send_empty_block(&[TEST_ENDPOINT_B.clone()], &com_hub).await;

    // TODO: validate that no block was sent?
}

#[async_test]
pub async fn send_block_to_multiple_endpoints() {
    let (com_hub_sections_sender, mut com_hub_sections_receiver) = create_unbounded_channel();

    let (com_hub, com_interface) = get_mock_setup_with_com_hub(MockupSetupData {
        interface_properties: MockupInterfaceSetupData {
            endpoint: None,
            ..Default::default()
        },
        com_hub_sections_sender: Some(com_hub_sections_sender),
        interface_priority: InterfacePriority::None,
        ..Default::default()
    }).await;

    let socket_uuid = com_interface.implementation_mut::<MockupInterface>().socket_uuid.clone();

    com_hub.socket_manager().borrow_mut().register_socket_endpoint(
        socket_uuid.clone(),
        TEST_ENDPOINT_A.clone(),
        1,
    );

    com_hub.socket_manager().borrow_mut().register_socket_endpoint(
        socket_uuid.clone(),
        TEST_ENDPOINT_B.clone(),
        1,
    );

    yield_now().await;

    // send block to multiple receivers
    let block = send_block_with_body(
        &[TEST_ENDPOINT_A.clone(), TEST_ENDPOINT_B.clone()],
        b"Hello world",
        &com_hub,
    )
    .await;


    // get last block that was sent
    let last_block = com_interface.implementation_mut::<MockupInterface>().last_block().unwrap();
    let block_bytes =
        DXBBlock::from_bytes(&last_block)
            .await
            .unwrap();

    assert_eq!(block_bytes.body, block.body);
}

#[async_test]
pub async fn send_blocks_to_multiple_endpoints() {

    let (outgoing_blocks_sender, mut outgoing_blocks_receiver) = create_unbounded_channel::<(Vec<u8>, ComInterfaceSocketUUID)>();

    let mut base_interface = BaseInterface::create(
        BaseInterfaceSetupData {
            properties: Default::default(),
            on_send_callback: Box::new(
                move |data: &[u8], socket_uuid| {
                    let data = data.to_vec();
                    let mut outgoing_blocks_sender = outgoing_blocks_sender.clone();
                    Box::pin(async move {
                        outgoing_blocks_sender.start_send((data, socket_uuid)).unwrap();
                        true
                    })
                },
            ),
        }
    );

    let com_hub = get_mock_setup_with_interface(
        base_interface.com_interface.clone(),
        TEST_ENDPOINT_ORIGIN.clone(),
        None,
        InterfacePriority::default(),
    );

    let socket_uuid_a = base_interface.register_new_socket_with_endpoint(
        InterfaceDirection::InOut,
        TEST_ENDPOINT_A.clone(),
    );

    let socket_uuid_b = base_interface.register_new_socket_with_endpoint(
        InterfaceDirection::InOut,
        TEST_ENDPOINT_B.clone(),
    );

    yield_now().await;

    // send block to multiple receivers
    let _ = send_empty_block(
        &[TEST_ENDPOINT_A.clone(), TEST_ENDPOINT_B.clone()],
        &com_hub,
    )
    .await;

    let (_, first_block_socket_uuid) = outgoing_blocks_receiver.next().await.unwrap();
    let (_, second_block_socket_uuid) = outgoing_blocks_receiver.next().await.unwrap();

    let socket_uuids = [first_block_socket_uuid, second_block_socket_uuid];
    assert!(socket_uuids.contains(&socket_uuid_a));
    assert!(socket_uuids.contains(&socket_uuid_b));
}

#[async_test]
pub async fn default_interface_create_socket_first() {
    let (com_hub, com_interface, ..) = get_default_mock_setup_with_com_hub().await;

    let _ = send_empty_block(
        std::slice::from_ref(&TEST_ENDPOINT_B),
        &com_hub,
    )
    .await;

    // sleep to let the com_hub process the new socket
    sleep(Duration::from_millis(10)).await;

    let mockup_interface =
        com_interface.implementation_mut::<MockupInterface>();
    assert_eq!(mockup_interface.outgoing_queue.borrow().len(), 1);
}

#[async_test]
pub async fn test_receive() {
    let (com_hub, com_interface, mut incoming_blocks_sender, mut incoming_sections_receiver) = get_default_mock_setup_with_com_hub().await;

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

    let block_bytes = block.to_bytes().unwrap();
    incoming_blocks_sender.start_send(block_bytes.as_slice().to_vec()).unwrap();

    yield_now().await;

    let last_block = get_next_received_single_block_from_receiver(&mut incoming_sections_receiver).await;
    assert_eq!(last_block.raw_bytes.clone().unwrap(), block_bytes);
}

#[async_test]
pub async fn unencrypted_signature_prepare_block_com_hub() {
    let (com_hub, com_interface, mut incoming_blocks_sender, mut incoming_sections_receiver) = get_default_mock_setup_with_com_hub().await;


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
    incoming_blocks_sender.start_send(block_bytes.as_slice().to_vec()).unwrap();


    yield_now().await;

    let last_block = get_next_received_single_block_from_receiver(&mut incoming_sections_receiver).await;
    assert_eq!(last_block.raw_bytes.clone().unwrap(), block_bytes);
    assert_eq!(block.signature, last_block.signature);

    assert!(com_hub.validate_block(&last_block).await.unwrap());
}

#[async_test]
pub async fn encrypted_signature_prepare_block_com_hub() {
    let (com_hub, com_interface, mut incoming_blocks_sender, mut incoming_sections_receiver) = get_default_mock_setup_with_com_hub().await;

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
    incoming_blocks_sender.start_send(block_bytes.as_slice().to_vec()).unwrap();

    yield_now().await;

    let last_block = get_next_received_single_block_from_receiver(&mut incoming_sections_receiver).await;
    assert_eq!(last_block.raw_bytes.clone().unwrap(), block_bytes);
    assert_eq!(block.signature, last_block.signature);

    assert!(com_hub.validate_block(&last_block).await.unwrap());
}

#[async_test]
pub async fn test_receive_multiple_blocks_single_section() {
    let (com_hub, com_interface, mut incoming_blocks_sender, mut incoming_sections_receiver) = get_default_mock_setup_with_com_hub().await;

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

    // send blocks via incoming_blocks_sender
    send_multiple_blocks_to_local(
        &mut incoming_blocks_sender,
        &mut blocks,
    ).await;

    // collect received blocks from incoming_sections_receiver
    let incoming_blocks = get_collected_received_blocks_from_receiver(
        &mut incoming_sections_receiver,
        CollectedBlockType::BlockStream,
        blocks_count
    ).await;

    for (incoming_block, block) in incoming_blocks.iter().zip(blocks.iter()) {
        assert_eq!(
            incoming_block.raw_bytes.clone().unwrap(),
            block.to_bytes().unwrap()
        );
    }
}



#[async_test]
pub async fn test_receive_multiple_separate_blocks() {
    let (com_hub, com_interface, mut incoming_blocks_sender, mut incoming_sections_receiver) = get_default_mock_setup_with_com_hub().await;

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

    // send blocks via incoming_blocks_sender
    send_multiple_blocks_to_local(
        &mut incoming_blocks_sender,
        &mut blocks,
    ).await;

    // collect received blocks from incoming_sections_receiver
    let incoming_blocks = get_collected_received_blocks_from_receiver(
        &mut incoming_sections_receiver,
        CollectedBlockType::SingleBocks,
        blocks_count
    ).await;

    for (incoming_block, block) in incoming_blocks.iter().zip(blocks.iter()) {
        assert_eq!(
            incoming_block.raw_bytes.clone().unwrap(),
            block.to_bytes().unwrap()
        );
    }
}

#[async_test]
pub async fn test_add_and_remove_interface_and_sockets() {
    let (com_hub, com_interface, mut incoming_blocks_sender, mut incoming_sections_receiver) = get_default_mock_setup_with_com_hub().await;

    {
        let interface_manager = com_hub.interface_manager();
        let socket_manager = com_hub.socket_manager();
        assert_eq!(interface_manager.borrow().interfaces.len(), 2); // loopback + mockup interface
        assert_eq!(socket_manager.borrow().sockets.len(), 2); // loopback + mockup socket
        assert_eq!(socket_manager.borrow().endpoint_sockets.len(), 2);
    }

    assert_eq!(com_interface.current_state(), ComInterfaceState::Connected);

    let socket_uuid = com_interface.implementation_mut::<MockupInterface>().socket_uuid.clone();

    let socket_state = {
        let socket_manager = com_hub.socket_manager();
        socket_manager.borrow().socket_state(&socket_uuid)
    };
    assert_eq!(socket_state, SocketState::Connected);

    let uuid = com_interface.uuid().clone();

    // remove interface
    assert!(com_hub.remove_interface(uuid).is_ok());

    {
        let interface_manager = com_hub.interface_manager();
        let socket_manager = com_hub.socket_manager();
        assert_eq!(interface_manager.borrow().interfaces.len(), 1); // loopback interface
        assert_eq!(socket_manager.borrow().sockets.len(), 1); // loopback socket
        assert_eq!(socket_manager.borrow().endpoint_sockets.len(), 1);
    }

    assert_eq!(com_interface.current_state(), ComInterfaceState::Destroyed);

    let socket_manager = com_hub.socket_manager();
    assert!(!socket_manager.borrow().has_socket(&socket_uuid))
}

#[async_test]
pub async fn test_basic_routing() {
    let (
        (com_hub_mut_a, ..),
        (.., mut incoming_sections_receiver_b)
    ) = get_default_mock_setup_with_two_connected_com_hubs().await;

    yield_now().await;
    yield_now().await;

    let block_a_to_b = send_block_with_body(
        std::slice::from_ref(&TEST_ENDPOINT_B),
        b"Hello world",
        &com_hub_mut_a,
    )
    .await;

    yield_now().await;

    let last_block = get_next_received_single_block_from_receiver(&mut incoming_sections_receiver_b).await;
    assert_eq!(block_a_to_b.body, last_block.body);
}

#[async_test]
pub async fn register_factory() {
    let com_hub = create_mock_com_hub();
    MockupInterface::register_on_com_hub(com_hub.clone());

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
    let mockup_interface = com_hub
        .create_interface(
            "mockup",
            to_value_container(&MockupInterfaceSetupData::new("mockup"))
                .unwrap(),
            InterfacePriority::default(),
        )
        .await
        .unwrap();

    assert_eq!(mockup_interface.properties().interface_type, "mockup");
}

#[async_test]
pub async fn test_reconnect() {
    let com_hub = create_mock_com_hub();

    // create a new interface, open it and add it to the com_hub
    let base_interface = BaseInterface::create(BaseInterfaceSetupData::new(
        InterfaceProperties {
            reconnection_config: ReconnectionConfig::ReconnectWithTimeout {
                timeout: Duration::from_secs(1),
            },
            ..InterfaceProperties::default()
        },
        Box::new(|_, _| Box::pin(async { true })),
    ));

    // add base_interface to com_hub
    com_hub.register_com_interface(
        base_interface.com_interface.clone(),
        InterfacePriority::default(),
    ).unwrap();

    // check that the interface is connected
    assert_eq!(
        base_interface.com_interface.current_state(),
        ComInterfaceState::Connected
    );

    // check that the interface is in the com_hub
    assert_eq!(com_hub.interface_manager().borrow().interfaces.len(), 2); // loopback + base_interface
    assert!(com_hub.has_interface(&base_interface.com_interface.uuid()));

    // simulate a disconnection by closing the interface
    // This action is normally done by the interface itself
    // but we do it manually here to test the reconnection
    base_interface.com_interface.close();

    // check that the interface is not connected
    // and that the close_timestamp is set
    assert_eq!(
        base_interface.com_interface.current_state(),
        ComInterfaceState::NotConnected
    );

    assert!(
        base_interface
            .com_interface
            .properties()
            .close_timestamp
            .is_some()
    );

    // the interface should not be reconnected yet
    yield_now().await;

    assert_eq!(
        base_interface.com_interface.current_state(),
        ComInterfaceState::NotConnected
    );

    // wait for the reconnection to happen
    tokio::time::sleep(Duration::from_secs(1)).await;

    // check that the interface is connected again
    // and that the close_timestamp is reset
    yield_now().await;

    assert_eq!(
        base_interface.com_interface.current_state(),
        ComInterfaceState::Connected
    );
}
