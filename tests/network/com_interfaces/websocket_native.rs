use std::assert_matches::assert_matches;
use std::time::Duration;
use datex_core::global::dxb_block::DXBBlock;
use datex_core::network::com_hub::errors::InterfaceCreateError;
use datex_core::task::sleep;
use datex_core::utils::context::init_global_context;
use datex_core::network::com_interfaces::default_com_interfaces::websocket::websocket_common::{WebSocketClientInterfaceSetupData, WebSocketError, WebSocketServerInterfaceSetupData};
use datex_core::network::com_interfaces::{
    default_com_interfaces::{
        websocket::websocket_client_native_interface::WebSocketClientNativeInterface,
        websocket::websocket_server_native_interface::WebSocketServerNativeInterface,
    },
};

use datex_core::network::com_interfaces::com_interface::ComInterface;
use datex_core::network::com_interfaces::com_interface::error::ComInterfaceError;
use datex_core::network::com_interfaces::com_interface::socket::ComInterfaceSocketEvent;
use datex_core::run_async;
use ntest_timeout::timeout;

#[tokio::test]
#[timeout(4000)]
pub async fn test_create_socket_connection() {
    run_async! {
        const PORT: u16 = 8085;
        init_global_context();

        let client_to_server_message = DXBBlock::new_with_body(b"Hello from client to server");
        let server_to_client_message = DXBBlock::new_with_body(b"Hello from server to client");

        let server_interface = ComInterface::create_async_with_implementation::<WebSocketServerNativeInterface>(
            WebSocketServerInterfaceSetupData {
                port: PORT,
                secure: Some(false),
            }
        ).await.expect("Failed to create WebSocketServerInterface");
        let mut server_interface_socket_event_receiver = server_interface.take_socket_event_receiver();

        let client_interface = ComInterface::create_async_with_implementation::<WebSocketClientNativeInterface>(
            WebSocketClientInterfaceSetupData {
                address: format!("ws://localhost:{PORT}")
            }
        ).await.expect("Failed to create WebSocketClientInterface");
        let mut client_interface_socket_event_receiver = client_interface.take_socket_event_receiver();

        // sockets must be connected, extract them from the event receivers
        let mut client_socket = match client_interface_socket_event_receiver.next().await {
            Some(ComInterfaceSocketEvent::NewSocket(socket)) => socket,
            _ => panic!("Expected NewSocket event for client"),
        };
        let mut client_socket_receiver = client_socket.take_block_in_receiver();
        let mut server_socket = match server_interface_socket_event_receiver.next().await {
            Some(ComInterfaceSocketEvent::NewSocket(socket)) => socket,
            _ => panic!("Expected NewSocket event for server"),
        };
        let mut server_socket_receiver = server_socket.take_block_in_receiver();

        // send block from client to server
        let client_uuid = client_interface.implementation::<WebSocketClientNativeInterface>().socket_uuid.clone();
        assert!(
            client_interface
                .send_block(&client_to_server_message.to_bytes().unwrap(), client_uuid)
                .await
        );

        // send block from server to client
        let server_socket_uuid = server_interface.implementation::<WebSocketServerNativeInterface>()
            .websocket_streams_by_socket
            .lock()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .clone();
        assert!(
            server_interface
                .send_block(&server_to_client_message.to_bytes().unwrap(), server_socket_uuid.clone())
                .await
        );

        sleep(Duration::from_millis(100)).await;

        // check if messages are received correctly
        assert_eq!(server_socket_receiver.next().await.unwrap(), client_to_server_message);
        assert_eq!(client_socket_receiver.next().await.unwrap(), server_to_client_message);
    }
}

#[tokio::test]
pub async fn test_construct_client() {
    init_global_context();

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
