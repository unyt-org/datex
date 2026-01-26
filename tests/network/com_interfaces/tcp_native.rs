use std::{assert_matches, sync::Arc};

use std::sync::Mutex;

use datex_core::{
    global::dxb_block::DXBBlock,
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::{
            com_interface::{ComInterface, socket::ComInterfaceSocketEvent},
            default_com_interfaces::tcp::tcp_common::{
                TCPClientInterfaceSetupData, TCPServerInterfaceSetupData,
            },
        },
    },
    runtime::AsyncContext,
};
use datex_macros::async_test;

#[async_test]
pub async fn test_client_no_connection() {
    assert_matches!(
        ComInterface::create_async_from_setup_data(
            TCPClientInterfaceSetupData {
                address: "0.0.0.0:9086".to_string(),
            },
            AsyncContext::default()
        )
        .await
        .unwrap_err(),
        ComInterfaceCreateError::InterfaceError(_)
    );
}

#[async_test]
pub async fn test_construct() {
    const PORT: u16 = 8088;
    let client_to_server_message =
        DXBBlock::new_with_body(b"Hello from client to server");
    let server_to_client_message =
        DXBBlock::new_with_body(b"Hello from server to client");

    let (server_interface, (_, mut server_interface_socket_event_receiver)) =
        ComInterface::create_async_from_setup_data(
            TCPServerInterfaceSetupData::new_with_port(PORT),
            AsyncContext::default(),
        )
        .await
        .unwrap();

    let (client_interface, (_, mut client_interface_socket_event_receiver)) =
        ComInterface::create_async_from_setup_data(
            TCPClientInterfaceSetupData {
                address: format!("0.0.0.0:{PORT}"),
            },
            AsyncContext::default(),
        )
        .await
        .unwrap();

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
    let client_socket_uuid = client_socket.uuid.clone();
    client_interface.send_block(
        client_to_server_message.clone(),
        client_socket_uuid.clone(),
    );

    // send block from server to client
    let server_socket_uuid = server_socket.uuid.clone();
    server_interface.send_block(
        server_to_client_message.clone(),
        server_socket_uuid.clone(),
    );

    // check if messages are received correctly
    assert_eq!(
        server_socket_receiver.next().await.unwrap(),
        client_to_server_message.clone()
    );
    assert_eq!(
        client_socket_receiver.next().await.unwrap(),
        server_to_client_message
    );

    // Parallel sending
    let client = Arc::new(Mutex::new(client_interface));
    for _ in 0..5 {
        let client_to_server_message_clone = client_to_server_message.clone();
        let client = client.clone();
        let client_uuid = client_socket_uuid.clone();
        client
            .try_lock()
            .unwrap()
            .send_block(client_to_server_message_clone, client_uuid.clone())
    }

    // We take ownership of the client
    let client = Arc::into_inner(client).unwrap();
    let client = Mutex::into_inner(client).unwrap();

    // FIXME
    // client.close().await;
    // server_interface.close().await;
}
