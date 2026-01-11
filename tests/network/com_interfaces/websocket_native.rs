use datex_core::{
    global::dxb_block::DXBBlock,
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::default_com_interfaces::websocket::{
            websocket_client_native_interface::WebSocketClientNativeInterface,
            websocket_common::{
                WebSocketClientInterfaceSetupData, WebSocketError,
                WebSocketServerInterfaceSetupData,
            },
            websocket_server_native_interface::WebSocketServerNativeInterface,
        },
    },
    task::sleep,
    utils::context::init_global_context,
};
use std::{assert_matches::assert_matches, time::Duration};

use datex_core::{
    network::com_interfaces::com_interface::{
        ComInterface, error::ComInterfaceError, socket::ComInterfaceSocketEvent,
    },
    run_async,
};
use datex_macros::async_test;
use ntest_timeout::timeout;

#[async_test]
#[timeout(4000)]
pub async fn test_create_socket_connection() {
    const PORT: u16 = 8085;

    let client_to_server_message =
        DXBBlock::new_with_body(b"Hello from client to server");
    let server_to_client_message =
        DXBBlock::new_with_body(b"Hello from server to client");

    let server_interface = ComInterface::create_async_with_implementation::<
        WebSocketServerNativeInterface,
    >(WebSocketServerInterfaceSetupData {
        port: PORT,
        secure: Some(false),
    })
    .await
    .expect("Failed to create WebSocketServerInterface");
    let mut server_interface_socket_event_receiver =
        server_interface.take_socket_event_receiver();

    let client_interface = ComInterface::create_async_with_implementation::<
        WebSocketClientNativeInterface,
    >(WebSocketClientInterfaceSetupData {
        address: format!("ws://localhost:{PORT}"),
    })
    .await
    .expect("Failed to create WebSocketClientInterface");
    let mut client_interface_socket_event_receiver =
        client_interface.take_socket_event_receiver();

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
    let client_uuid = client_interface
        .implementation::<WebSocketClientNativeInterface>()
        .socket_uuid
        .clone();
    client_interface
        .send_block(&client_to_server_message.to_bytes().unwrap(), client_uuid);

    // send block from server to client
    let server_socket_uuid = server_interface
        .implementation::<WebSocketServerNativeInterface>()
        .websocket_streams_by_socket
        .lock()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    server_interface.send_block(
        &server_to_client_message.to_bytes().unwrap(),
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
    let client_res = ComInterface::create_async_with_implementation::<
        WebSocketClientNativeInterface,
    >(WebSocketClientInterfaceSetupData {
        address: "ftp://localhost:1234".to_string(),
    })
    .await;
    assert_matches!(
        client_res.unwrap_err(),
        InterfaceCreateError::InvalidSetupData(_)
    );

    // We expect a connection error here, as the server can't be reached
    let client_res = ComInterface::create_async_with_implementation::<
        WebSocketClientNativeInterface,
    >(WebSocketClientInterfaceSetupData {
        address: "ws://localhost.invalid:1234".to_string(),
    })
    .await;

    assert_matches!(
        client_res.unwrap_err(),
        InterfaceCreateError::InterfaceError(
            ComInterfaceError::ConnectionError(_)
        )
    );
}
