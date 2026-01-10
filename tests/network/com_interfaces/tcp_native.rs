use std::sync::{Arc, Mutex};

use datex_core::network::com_interfaces::default_com_interfaces::tcp::{
    tcp_client_native_interface::TCPClientNativeInterface,
    tcp_common::TCPError,
    tcp_server_native_interface::TCPServerNativeInterface,
};
use datex_core::run_async;
use datex_core::utils::context::init_global_context;
use futures::future::join_all;
use datex_core::network::com_interfaces::com_interface::ComInterface;
use datex_core::network::com_interfaces::default_com_interfaces::tcp::tcp_common::{TCPClientInterfaceSetupData, TCPServerInterfaceSetupData};

#[tokio::test]
pub async fn test_client_no_connection() {
    init_global_context();
    let mut client_interface = ComInterface::create_sync_with_implementation::<TCPClientNativeInterface>(
        TCPClientInterfaceSetupData { address: "0.0.0.0:8080".to_string()}
    ).unwrap();

    assert!(client_interface.state().lock().unwrap().get().is_not_connected());
    let res = client_interface.reconnect().await;
    assert_eq!(res, false);
    assert_eq!(res.unwrap_err(), TCPError::ConnectionError);
    assert!(client_interface.state().lock().unwrap().get().is_not_connected());
    client_interface.handle_destroy().await;
}

#[tokio::test]
pub async fn test_construct() {
    run_async! {
        const PORT: u16 = 8088;
        const CLIENT_TO_SERVER_MSG: &[u8] = b"Hello World";
        const SERVER_TO_CLIENT_MSG: &[u8] = b"Nooo, this is Patrick!";

        init_global_context();

        // let mut server = TCPServerNativeInterface::new(PORT).unwrap();
        let mut server_interface = ComInterface::create_sync_with_implementation::<TCPServerNativeInterface>(
            TCPServerInterfaceSetupData { port: PORT }
        ).unwrap();

        assert_eq!(server_interface.reconnect().await, true);

        let mut client_interface = ComInterface::create_sync_with_implementation::<TCPClientNativeInterface>(
            TCPClientInterfaceSetupData { address: format!("0.0.0.0:{PORT}") }
        ).unwrap();

        let client_uuid = client_interface.implementation::<TCPClientNativeInterface>().uuid;

        assert!(
            client_interface
                .send_block(CLIENT_TO_SERVER_MSG, client_uuid.clone())
                .await
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        let server_uuid = server_interface.implementation()
        assert!(
            server_interface
                .send_block(SERVER_TO_CLIENT_MSG, server_uuid.clone())
                .await
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        // Check if the client received the message
        // FIXME update loop
        // assert_eq!(
        //     client
        //         .get_socket()
        //         .unwrap()
        //         .try_lock()
        //         .unwrap()
        //         .receive_queue
        //         .try_lock()
        //         .unwrap()
        //         .drain(..)
        //         .collect::<Vec<_>>(),
        //     SERVER_TO_CLIENT_MSG
        // );

        {
            // Check if the server received the message
            let server_socket = server_interface.get_socket_with_uuid(server_uuid).unwrap();
            // FIXME update loop
            // assert_eq!(
            //     server_socket
            //         .try_lock()
            //         .unwrap()
            //         .receive_queue
            //         .try_lock()
            //         .unwrap()
            //         .drain(..)
            //         .collect::<Vec<_>>(),
            //     CLIENT_TO_SERVER_MSG
            // );
        }

        // Parallel sending
        let client = Arc::new(Mutex::new(client_interface));
        let mut futures = vec![];
        for _ in 0..5 {
            let client = client.clone();
            let client_uuid = client_uuid.clone();
            futures.push(async move {
                client
                    .try_lock()
                    .unwrap()
                    .send_block(CLIENT_TO_SERVER_MSG, client_uuid.clone())
                    .await;
            });
        }
        join_all(futures).await;

        // We take ownership of the client
        let client = Arc::into_inner(client).unwrap();
        let client = Mutex::into_inner(client).unwrap();
        client.destroy().await;

        server_interface.destroy().await;
    }
}
