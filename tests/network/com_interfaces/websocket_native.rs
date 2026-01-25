use datex_core::{
    global::dxb_block::DXBBlock,
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::default_com_interfaces::websocket::websocket_common::{
            WebSocketClientInterfaceSetupData,
            WebSocketServerInterfaceSetupData,
        },
    },
    task::sleep,
};
use std::{assert_matches, time::Duration};

use datex_core::{
    network::com_interfaces::{
        com_interface::{
            ComInterface, error::ComInterfaceError,
            socket::ComInterfaceSocketEvent,
        },
        default_com_interfaces::websocket::websocket_common::TLSMode,
    },
    run_async,
    runtime::AsyncContext,
    serde::deserializer::from_value_container,
};
use datex_macros::async_test;

#[async_test]
#[timeout(4000)]
pub async fn test_create_socket_connection() {
    const ADDRESS: &str = "0.0.0.0:8085";

    let client_to_server_message =
        DXBBlock::new_with_body(b"Hello from client to server");
    let server_to_client_message =
        DXBBlock::new_with_body(b"Hello from server to client");

    let (server_interface, (_, mut server_interface_socket_event_receiver)) =
        ComInterface::create_async_from_setup_data(
            WebSocketServerInterfaceSetupData {
                bind_address: ADDRESS.to_string(),
                accept_addresses: None,
            },
            AsyncContext::default(),
        )
        .await
        .expect("Failed to create WebSocketServerInterface");

    let (client_interface, (_, mut client_interface_socket_event_receiver)) =
        ComInterface::create_async_from_setup_data(
            WebSocketClientInterfaceSetupData {
                url: format!("ws://{ADDRESS}"),
            },
            AsyncContext::default(),
        )
        .await
        .expect("Failed to create WebSocketClientInterface");

    // sockets must be connected, extract them from the event receivers
    let mut client_socket =
        match client_interface_socket_event_receiver.next().await {
            Some(ComInterfaceSocketEvent::NewSocket(socket)) => socket,
            _ => panic!("Expected NewSocket event for client"),
        };
    let mut client_socket_receiver = client_socket.take_block_in_receiver();
    let mut server_socket =
        match server_interface_socket_event_receiver.next().await {
            Some(ComInterfaceSocketEvent::NewSocket(socket)) => socket,
            _ => panic!("Expected NewSocket event for server"),
        };
    let mut server_socket_receiver = server_socket.take_block_in_receiver();

    // send block from client to server
    let client_uuid = client_socket.uuid.clone();
    client_interface.send_block(client_to_server_message.clone(), client_uuid);

    // send block from server to client
    let server_socket_uuid = server_socket.uuid.clone();
    server_interface.send_block(
        server_to_client_message.clone(),
        server_socket_uuid.clone(),
    );

    sleep(Duration::from_millis(100)).await;

    // check if messages are received correctly
    assert_eq!(
        server_socket_receiver.next().await.unwrap(),
        client_to_server_message
    );
    assert_eq!(
        client_socket_receiver.next().await.unwrap(),
        server_to_client_message
    );
}

#[async_test]
pub async fn test_construct_client() {
    // Test with a invalid URL
    let client_res = ComInterface::create_async_from_setup_data(
        WebSocketClientInterfaceSetupData {
            url: "ftp://localhost:1234".to_string(),
        },
        AsyncContext::default(),
    )
    .await;
    assert_matches!(
        client_res.unwrap_err(),
        InterfaceCreateError::InvalidSetupData(_)
    );

    // We expect a connection error here, as the server can't be reached
    let client_res = ComInterface::create_async_from_setup_data(
        WebSocketClientInterfaceSetupData {
            url: "ws://localhost.invalid:1234".to_string(),
        },
        AsyncContext::default(),
    )
    .await;

    assert_matches!(
        client_res.unwrap_err(),
        InterfaceCreateError::InterfaceError(
            ComInterfaceError::ConnectionError(_)
        )
    );
}

#[async_test]
pub async fn test_connectable_interface_config() {
    const ADDRESS: &str = "0.0.0.0:8086";
    let accept_addresses = vec![
        ("example.com".to_string(), Some(TLSMode::HandledExternally)),
        (
            "example.org:9000".to_string(),
            Some(TLSMode::WithCertificate {
                certificate: vec![],
                private_key: vec![],
            }),
        ),
        ("localhost:8086".to_string(), None),
    ];

    let server_setup_data = WebSocketServerInterfaceSetupData {
        bind_address: ADDRESS.to_string(),
        accept_addresses: Some(accept_addresses.clone()),
    };
    let (server_interface, ..) = ComInterface::create_async_from_setup_data(
        server_setup_data,
        AsyncContext::default(),
    )
    .await
    .unwrap();

    let properties = server_interface.properties.borrow();
    let connectable_interfaces =
        properties.connectable_interfaces.as_ref().unwrap();

    for (config, accept_address) in
        connectable_interfaces.iter().zip(accept_addresses.iter())
    {
        assert_eq!(config.interface_type, "websocket-client");
        let setup_data = config
            .setup_data
            .clone()
            .cast_to::<WebSocketClientInterfaceSetupData>()
            .unwrap();
        assert_eq!(
            setup_data.url,
            format!(
                "{}://{}",
                if accept_address.1.is_some() {
                    "wss"
                } else {
                    "ws"
                },
                accept_address.0
            )
        );
    }
}
