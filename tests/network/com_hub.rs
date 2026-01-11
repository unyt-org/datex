use datex_core::serde::serializer::to_value_container;
use datex_core::global::dxb_block::DXBBlock;
use datex_core::global::protocol_structures::block_header::BlockHeader;
use datex_core::global::protocol_structures::encrypted_header::{
    self, EncryptedHeader,
};
use datex_core::global::protocol_structures::routing_header::{RoutingHeader, SignatureType};
use datex_core::network::com_hub::{ComHub, InterfacePriority};
use datex_core::network::com_interfaces::default_com_interfaces::base_interface::{BaseInterface, BaseInterfaceHolder, BaseInterfaceSetupData};
use datex_core::stdlib::cell::RefCell;
use datex_core::stdlib::rc::Rc;
use datex_macros::async_test;
use std::pin::Pin;
use std::sync::mpsc;
use tokio::task::yield_now;
use datex_core::network::block_handler::IncomingSectionsSinkType;
use datex_core::network::com_interfaces::com_interface::ComInterface;
use datex_core::network::com_interfaces::com_interface::implementation::ComInterfaceSyncFactory;
use datex_core::network::com_interfaces::com_interface::properties::{InterfaceProperties, ReconnectionConfig};
use datex_core::network::com_interfaces::com_interface::socket::SocketState;
use datex_core::network::com_interfaces::com_interface::state::ComInterfaceState;
use super::helpers::mock_setup::get_mock_setup_and_socket_for_endpoint;
use datex_core::utils::context::init_global_context;
use crate::network::helpers::mock_setup::{
    TEST_ENDPOINT_A, TEST_ENDPOINT_B, TEST_ENDPOINT_ORIGIN, create_and_add_socket,
    get_all_received_single_blocks_from_com_hub,
    get_last_received_single_block_from_com_hub, get_mock_setup,
    get_mock_setup_and_socket, get_mock_setup_and_socket_for_priority,
    get_mock_setup_with_endpoint, register_socket_endpoint,
    send_block_with_body, send_empty_block_and_update,
};
use crate::network::helpers::mockup_interface::{
    MockupInterface, MockupInterfaceSetupData,
};
use datex_core::runtime::AsyncContext;
use datex_core::values::core_values::endpoint::Endpoint;

#[async_test]
pub async fn test_add_and_remove() {
    let com_hub = Rc::new(ComHub::create(
        Endpoint::default(),
        AsyncContext::new(),
        IncomingSectionsSinkType::Channel,
    ));
    let uuid = {
        let mockup_interface = ComInterface::create_sync_with_implementation::<
            MockupInterface,
        >(MockupInterfaceSetupData::new("test"))
        .unwrap();
        let uuid = mockup_interface.uuid().clone();
        com_hub.register_com_interface(
            mockup_interface,
            InterfacePriority::default(),
        );
        uuid
    };
    assert!(com_hub.remove_interface(uuid).await.is_ok());
}

#[async_test]
pub async fn test_multiple_add() {
    let com_hub = ComHub::create(
        Endpoint::default(),
        AsyncContext::new(),
        IncomingSectionsSinkType::Collector,
    );

    let mockup_interface1 =
        ComInterface::create_sync_with_implementation::<MockupInterface>(
            MockupInterfaceSetupData::new("mockup_interface1"),
        )
        .unwrap();
    let mockup_interface2 =
        ComInterface::create_sync_with_implementation::<MockupInterface>(
            MockupInterfaceSetupData::new("mockup_interface2"),
        )
        .unwrap();

    com_hub.register_com_interface(
        mockup_interface1.clone(),
        InterfacePriority::default(),
    );
    com_hub.register_com_interface(
        mockup_interface2.clone(),
        InterfacePriority::default(),
    );

    panic!("fixme")
    // assert!(
    //     com_hub
    //         .register_com_interface(
    //             mockup_interface1.clone(),
    //             InterfacePriority::default()
    //         )
    //         .await
    //         .is_err()
    // );
    // assert!(
    //     com_hub
    //         .register_com_interface(
    //             mockup_interface2.clone(),
    //             InterfacePriority::default()
    //         )
    //         .await
    //         .is_err()
    // );
}

#[async_test]
pub async fn test_send() {
    let (com_hub, com_interface, _) =
        get_mock_setup_and_socket(IncomingSectionsSinkType::Channel).await;

    let block = send_block_with_body(
        &[TEST_ENDPOINT_A.clone()],
        b"Hello world!",
        &com_hub,
    )
    .await;

    // get last block that was sent
    let mockup_interface_out =
        com_interface.implementation_mut::<MockupInterface>();
    let block_bytes =
        DXBBlock::from_bytes(&mockup_interface_out.last_block().unwrap())
            .await
            .unwrap();

    assert!(mockup_interface_out.last_block().is_some());
    assert_eq!(block_bytes.body, block.body);
}

#[async_test]
pub async fn test_send_invalid_recipient() {
    // init without fallback interfaces
    let (com_hub, com_interface, _) = get_mock_setup_and_socket_for_priority(
        InterfacePriority::None,
        IncomingSectionsSinkType::Channel,
    )
    .await;

    send_empty_block_and_update(&[TEST_ENDPOINT_B.clone()], &com_hub).await;

    // get last block that was sent
    let mockup_interface_out =
        com_interface.implementation_mut::<MockupInterface>();
    assert!(mockup_interface_out.last_block().is_none());
}

#[async_test]
pub async fn send_block_to_multiple_endpoints() {
    let (com_hub, com_interface) = get_mock_setup().await;
    let socket_uuid = {
        let mut mockup_interface =
            com_interface.implementation_mut::<MockupInterface>();
        create_and_add_socket(&mut mockup_interface).unwrap()
    };
    register_socket_endpoint(
        com_interface.clone(),
        socket_uuid.clone(),
        TEST_ENDPOINT_A.clone(),
    );
    register_socket_endpoint(
        com_interface.clone(),
        socket_uuid.clone(),
        TEST_ENDPOINT_B.clone(),
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
    let mockup_interface =
        com_interface.implementation_mut::<MockupInterface>();
    let block_bytes =
        DXBBlock::from_bytes(&mockup_interface.last_block().unwrap())
            .await
            .unwrap();

    assert_eq!(mockup_interface.outgoing_queue.borrow().len(), 1);
    assert!(mockup_interface.last_block().is_some());
    assert_eq!(block_bytes.body, block.body);
}

#[async_test]
pub async fn send_blocks_to_multiple_endpoints() {
    let (com_hub, com_interface) = get_mock_setup().await;

    let (socket_uuid_a, socket_uuid_b) = {
        let mut mockup_interface =
            com_interface.implementation_mut::<MockupInterface>();
        (
            create_and_add_socket(&mut mockup_interface).unwrap(),
            create_and_add_socket(&mut mockup_interface).unwrap(),
        )
    };

    register_socket_endpoint(
        com_interface.clone(),
        socket_uuid_a.clone(),
        TEST_ENDPOINT_A.clone(),
    );
    register_socket_endpoint(
        com_interface.clone(),
        socket_uuid_b.clone(),
        TEST_ENDPOINT_B.clone(),
    );
    yield_now().await;

    // send block to multiple receivers
    let _ = send_empty_block_and_update(
        &[TEST_ENDPOINT_A.clone(), TEST_ENDPOINT_B.clone()],
        &com_hub,
    )
    .await;

    let mockup_interface =
        com_interface.implementation_mut::<MockupInterface>();
    assert_eq!(mockup_interface.outgoing_queue.borrow().len(), 2);

    assert!(mockup_interface.has_outgoing_block_for_socket(&socket_uuid_a));
    assert!(mockup_interface.has_outgoing_block_for_socket(&socket_uuid_b));

    assert!(mockup_interface.last_block().is_some());
}

#[async_test]
pub async fn default_interface_create_socket_first() {
    let (com_hub, com_interface, _) = get_mock_setup_and_socket_for_priority(
        InterfacePriority::default(),
        IncomingSectionsSinkType::Channel,
    )
    .await;

    let _ = send_empty_block_and_update(
        std::slice::from_ref(&TEST_ENDPOINT_B),
        &com_hub,
    )
    .await;

    let mockup_interface =
        com_interface.implementation_mut::<MockupInterface>();
    assert_eq!(mockup_interface.outgoing_queue.borrow().len(), 1);
}

#[async_test]
pub async fn default_interface_set_default_interface_first() {
    let (com_hub, com_interface) = get_mock_setup_with_endpoint(
        TEST_ENDPOINT_ORIGIN.clone(),
        InterfacePriority::default(),
        IncomingSectionsSinkType::Collector,
    )
    .await;

    let socket_uuid = {
        let mut mockup_interface =
            com_interface.implementation_mut::<MockupInterface>();
        create_and_add_socket(&mut mockup_interface).unwrap()
    };

    register_socket_endpoint(
        com_interface.clone(),
        socket_uuid.clone(),
        TEST_ENDPOINT_A.clone(),
    );

    // Update to let the com_hub know about the socket and call the add_socket method
    // This will set the default interface and socket
    yield_now().await;
    let _ = send_empty_block_and_update(
        core::slice::from_ref(&TEST_ENDPOINT_B),
        &com_hub,
    )
    .await;

    let mockup_interface =
        com_interface.implementation_mut::<MockupInterface>();
    assert_eq!(mockup_interface.outgoing_queue.borrow().len(), 1);
}

#[async_test]
pub async fn test_receive() {
    let (com_hub, com_interface, socket_uuid) =
        get_mock_setup_and_socket(IncomingSectionsSinkType::Collector).await;

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
    {
        let mockup_interface =
            com_interface.implementation_mut::<MockupInterface>();
        let mut mockup_interface = mockup_interface.socket_senders.borrow_mut();
        let sender = mockup_interface.get_mut(&socket_uuid).unwrap();
        sender.start_send(block_bytes.as_slice().to_vec()).unwrap();
    }

    yield_now().await;

    let last_block = get_last_received_single_block_from_com_hub(&com_hub);
    assert_eq!(last_block.raw_bytes.clone().unwrap(), block_bytes);
}

#[async_test]
pub async fn unencrypted_signature_prepare_block_com_hub() {
    let (com_hub, com_interface, socket_uuid) =
        get_mock_setup_and_socket(IncomingSectionsSinkType::Collector).await;

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
    {
        let mockup_interface =
            com_interface.implementation_mut::<MockupInterface>();
        let mut mockup_interface = mockup_interface.socket_senders.borrow_mut();
        let sender = mockup_interface.get_mut(&socket_uuid).unwrap();
        sender.start_send(block_bytes.as_slice().to_vec()).unwrap();
    }

    yield_now().await;

    let last_block = get_last_received_single_block_from_com_hub(&com_hub);
    assert_eq!(last_block.raw_bytes.clone().unwrap(), block_bytes);
    assert_eq!(block.signature, last_block.signature);

    assert!(com_hub.validate_block(&last_block).await.unwrap());
}

#[async_test]
pub async fn encrypted_signature_prepare_block_com_hub() {
    let (com_hub, com_interface, socket_uuid) =
        get_mock_setup_and_socket(IncomingSectionsSinkType::Collector).await;

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
    {
        let mockup_interface =
            com_interface.implementation_mut::<MockupInterface>();
        let mut mockup_interface = mockup_interface.socket_senders.borrow_mut();
        let sender = mockup_interface.get_mut(&socket_uuid).unwrap();
        sender.start_send(block_bytes.as_slice().to_vec()).unwrap();
    }
    yield_now().await;

    let last_block = get_last_received_single_block_from_com_hub(&com_hub);
    assert_eq!(last_block.raw_bytes.clone().unwrap(), block_bytes);
    assert_eq!(block.signature, last_block.signature);

    assert!(com_hub.validate_block(&last_block).await.unwrap());
}

#[async_test]
pub async fn test_receive_multiple() {
    let (com_hub, com_interface, socket_uuid) =
        get_mock_setup_and_socket(IncomingSectionsSinkType::Collector).await;

    // receive block
    let mut blocks = vec![
        DXBBlock {
            routing_header: RoutingHeader::default(),
            block_header: BlockHeader {
                section_index: 0,
                ..Default::default()
            },
            ..Default::default()
        },
        DXBBlock {
            routing_header: RoutingHeader::default(),
            block_header: BlockHeader {
                section_index: 1,
                ..Default::default()
            },
            ..Default::default()
        },
        DXBBlock {
            routing_header: RoutingHeader::default(),
            block_header: BlockHeader {
                section_index: 2,
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    for block in &mut blocks {
        // set receiver to ORIGIN
        block.set_receivers(vec![TEST_ENDPOINT_ORIGIN.clone()]);
    }

    let block_bytes: Vec<Vec<u8>> = blocks
        .iter()
        .map(|block| block.to_bytes().unwrap())
        .collect();

    {
        let mockup_interface =
            com_interface.implementation_mut::<MockupInterface>();
        let mut mockup_interface = mockup_interface.socket_senders.borrow_mut();
        let sender = mockup_interface.get_mut(&socket_uuid).unwrap();

        for block in block_bytes.into_iter() {
            sender.start_send(block).unwrap();
        }
    }

    yield_now().await;

    let incoming_blocks = get_all_received_single_blocks_from_com_hub(&com_hub);

    assert_eq!(incoming_blocks.len(), blocks.len());

    for (incoming_block, block) in incoming_blocks.iter().zip(blocks.iter()) {
        assert_eq!(
            incoming_block.raw_bytes.clone().unwrap(),
            block.to_bytes().unwrap()
        );
    }
}

#[async_test]
pub async fn test_add_and_remove_interface_and_sockets() {
    let (com_hub, com_interface, socket_uuid) =
        get_mock_setup_and_socket(IncomingSectionsSinkType::Collector).await;

    {
        let interface_manager = com_hub.interface_manager();
        let socket_manager = com_hub.socket_manager();
        assert_eq!(interface_manager.borrow().interfaces.len(), 2); // loopback + mockup interface
        // FIXME: should be 2 sockets, but loopback socket is not correctly initialized
        assert_eq!(socket_manager.borrow().sockets.len(), 2); // loopback + mockup socket
        assert_eq!(socket_manager.borrow().endpoint_sockets.len(), 2);
    }

    assert_eq!(com_interface.current_state(), ComInterfaceState::Connected);

    let socket_state = {
        let socket_manager = com_hub.socket_manager();
        socket_manager.borrow().socket_state(&socket_uuid)
    };
    assert_eq!(socket_state, SocketState::Connected);

    let uuid = com_interface.uuid().clone();

    // remove interface
    assert!(com_hub.remove_interface(uuid).await.is_ok());

    {
        let interface_manager = com_hub.interface_manager();
        let socket_manager = com_hub.socket_manager();
        assert_eq!(interface_manager.borrow().interfaces.len(), 1); // loopback interface
        assert_eq!(socket_manager.borrow().sockets.len(), 1); // loopback socket
        assert_eq!(socket_manager.borrow().endpoint_sockets.len(), 1);
    }

    assert_eq!(com_interface.current_state(), ComInterfaceState::Destroyed);

    let socket_state = {
        let socket_manager = com_hub.socket_manager();
        socket_manager.borrow().socket_state(&socket_uuid)
    };
    assert_eq!(socket_state, SocketState::Disconnected);
}

#[async_test]
pub async fn test_basic_routing() {
    let (sender_a, receiver_a) = mpsc::channel::<Vec<u8>>();
    let (sender_b, receiver_b) = mpsc::channel::<Vec<u8>>();

    let (com_hub_mut_a, com_interface_a, socket_a) =
        get_mock_setup_and_socket_for_endpoint(
            TEST_ENDPOINT_A.clone(),
            None,
            Some(sender_a),
            Some(receiver_b),
            InterfacePriority::default(),
            IncomingSectionsSinkType::Channel,
        )
        .await;

    let (com_hub_mut_b, com_interface_b, socket_b) =
        get_mock_setup_and_socket_for_endpoint(
            TEST_ENDPOINT_B.clone(),
            None,
            Some(sender_b),
            Some(receiver_a),
            InterfacePriority::default(),
            IncomingSectionsSinkType::Collector,
        )
        .await;

    {
        let mut mockup_interface_a =
            com_interface_a.implementation_mut::<MockupInterface>();
        mockup_interface_a.update();

        let mut mockup_interface_b =
            com_interface_b.implementation_mut::<MockupInterface>();
        mockup_interface_b.update();
    }

    yield_now().await;
    yield_now().await;

    let block_a_to_b = send_block_with_body(
        std::slice::from_ref(&TEST_ENDPOINT_B),
        b"Hello world",
        &com_hub_mut_a,
    )
    .await;

    {
        let mut mockup_interface_b =
            com_interface_b.implementation_mut::<MockupInterface>();
        mockup_interface_b.update();
    }
    yield_now().await;

    let last_block =
        get_last_received_single_block_from_com_hub(&com_hub_mut_b);
    assert_eq!(block_a_to_b.body, last_block.body);
}

#[async_test]
pub async fn register_factory() {
    let com_hub = ComHub::create(
        Endpoint::default(),
        AsyncContext::new(),
        IncomingSectionsSinkType::Collector,
    );
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
    let com_hub = ComHub::create(
        Endpoint::default(),
        AsyncContext::new(),
        IncomingSectionsSinkType::Channel,
    );

    // create a new interface, open it and add it to the com_hub
    let base_interface = BaseInterfaceHolder::new(BaseInterfaceSetupData::new(
        InterfaceProperties {
            reconnection_config: ReconnectionConfig::ReconnectWithTimeout {
                timeout: core::time::Duration::from_secs(1),
            },
            ..InterfaceProperties::default()
        },
        Box::new(|_, _| Box::pin(async { true })),
    ));

    // add base_interface to com_hub
    com_hub.register_com_interface(
        base_interface.com_interface.clone(),
        InterfacePriority::default(),
    );

    // check that the interface is connected
    assert_eq!(
        base_interface.com_interface.current_state(),
        ComInterfaceState::Connected
    );

    // check that the interface is in the com_hub
    assert_eq!(com_hub.interface_manager().borrow().interfaces.len(), 1);
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
    tokio::time::sleep(core::time::Duration::from_secs(1)).await;

    // check that the interface is connected again
    // and that the close_timestamp is reset
    yield_now().await;

    assert_eq!(
        base_interface.com_interface.current_state(),
        ComInterfaceState::Connected
    );
}
