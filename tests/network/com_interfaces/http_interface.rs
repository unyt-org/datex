use core::str::FromStr;
use axum::http;
use datex_core::{network::com_interfaces::{
    com_interface::socket::ComInterfaceSocketEvent, default_com_interfaces::http::http_server_interface::HTTPServerNativeInterface
}, run_async, values::core_values::endpoint::Endpoint};
use datex_core::network::com_interfaces::com_interface::ComInterface;
use datex_core::network::com_interfaces::default_com_interfaces::http::http_common::HTTPServerInterfaceSetupData;
use datex_core::utils::context::init_global_context;

// $ head -c 48192 /dev/zero | curl -X POST http://localhost:8081/my-secret-channel/tx --data-binary @-
#[tokio::test]
pub async fn test_construct() {
    run_async! {
        const PORT: u16 = 8081;
        init_global_context();

        let server = ComInterface::create_async_with_implementation::<
            HTTPServerNativeInterface,
        >(HTTPServerInterfaceSetupData { port: PORT })
        .await
        .expect("Failed to create HTTP server interface");

        let endpoint = Endpoint::from_str("@jonas").unwrap();

        let mut http_server_interface =
            server.implementation_mut::<HTTPServerNativeInterface>();
        http_server_interface
            .add_channel("my-secret-channel", endpoint.clone())
            .await;
        drop(http_server_interface);
        let mut socket_event_receiver = server.take_socket_event_receiver();
        let socket_uuid = match socket_event_receiver.next().await.unwrap() {
            ComInterfaceSocketEvent::NewSocket(socket) => socket.uuid,
            _ => panic!("Expected SocketCreated event"),
        };
        let mut it = 0;

        while it < 5 {
            server.send_block(b"Hello World", socket_uuid.clone());
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            it += 1;
        }

        let mut http_server_interface =
            server.implementation_mut::<HTTPServerNativeInterface>();
        http_server_interface
            .remove_channel("my-secret-channel")
            .await;
        drop(http_server_interface);
        server.close();
    }
}
