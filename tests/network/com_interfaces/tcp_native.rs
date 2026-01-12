use std::{
    assert_matches::assert_matches,
    sync::{Arc, Mutex},
};

use datex_core::{
    global::dxb_block::DXBBlock,
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::{
            com_interface::{ComInterface, socket::ComInterfaceSocketEvent},
            default_com_interfaces::tcp::{
                tcp_client_native_interface::TCPClientNativeInterface,
                tcp_common::{
                    TCPClientInterfaceSetupData, TCPServerInterfaceSetupData,
                },
                tcp_server_native_interface::TCPServerNativeInterface,
            },
        },
    },
    utils::context::init_global_context,
};
use datex_macros::async_test;
use futures::future::join_all;

#[async_test]
pub async fn test_client_no_connection() {
    assert_matches!(
        ComInterface::create_async_from_setup_data::<
            TCPClientNativeInterface,
        >(TCPClientInterfaceSetupData {
            address: "0.0.0.0:9086".to_string(),
        })
        .await
        .unwrap_err(),
        InterfaceCreateError::InterfaceError(_)
    );
}

#[async_test]
pub async fn test_construct() {
    const PORT: u16 = 8088;
    let client_to_server_message =
        DXBBlock::new_with_body(b"Hello from client to server");
    let server_to_client_message =
        DXBBlock::new_with_body(b"Hello from server to client");

    // let mut server = TCPServerNativeInterface::new(PORT).unwrap();
    let server_interface =
        ComInterface::create_async_from_setup_data::<
            TCPServerNativeInterface,
        >(TCPServerInterfaceSetupData::new_with_port(PORT))
        .await
        .unwrap();
    let mut server_interface_socket_event_receiver =
        server_interface.take_socket_event_receiver();

    let client_interface = ComInterface::create_async_from_setup_data::<
        TCPClientNativeInterface,
    >(TCPClientInterfaceSetupData {
        address: format!("0.0.0.0:{PORT}"),
    })
    .await
    .unwrap();

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
        .implementation::<TCPClientNativeInterface>()
        .socket_uuid
        .clone();
    client_interface.send_block(
        &client_to_server_message.to_bytes().unwrap(),
        client_uuid.clone(),
    );

    // send block from server to client
    let server_socket_uuid = server_interface
        .implementation::<TCPServerNativeInterface>()
        .tx_by_socket
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

    // check if messages are received correctly
    assert_eq!(
        server_socket_receiver.next().await.unwrap(),
        client_to_server_message
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
        let client_uuid = client_uuid.clone();
        client.try_lock().unwrap().send_block(
            &client_to_server_message_clone.to_bytes().unwrap(),
            client_uuid.clone(),
        )
    }

    // We take ownership of the client
    let client = Arc::into_inner(client).unwrap();
    let client = Mutex::into_inner(client).unwrap();

    // FIXME
    // client.close().await;
    // server_interface.close().await;
}
