pub mod http_server;
pub mod http_client;

#[cfg(test)]
mod tests {
    use ntest_timeout::timeout;
    use tokio::join;
    use datex::global::dxb_block::DXBBlock;
    use datex::native::com_interfaces::http::http_client::HTTPClientInterfaceSetupDataNative;
    use datex::native::com_interfaces::http::http_server::HTTPServerInterfaceSetupDataNative;
    use datex::native::com_interfaces::tests::{test_receive_block, test_send_block_async_callback, test_send_block_sync_once_callback};
    use datex::native_global_context::init_global_context_native;
    use datex::network::com_interfaces::com_interface::factory::{ComInterfaceAsyncFactory};

    use datex::network::com_interfaces::default_setup_data::http::http_client::HTTPClientInterfaceSetupData;
    use datex::network::com_interfaces::default_setup_data::http::http_server::HTTPServerInterfaceSetupData;
    use datex::utils::async_iterators::async_next_pin_box;

    #[tokio::test]
    #[timeout(100000)]
    async fn test_connect_and_communicate() {
        init_global_context_native();
        let address= "0.0.0.0:45679".to_string();

        let mut server_interface_configuration =
            HTTPServerInterfaceSetupDataNative(HTTPServerInterfaceSetupData {
                bind_address: address.clone(),
                accept_addresses: None,
            })
                .create_interface()
                .await
                .unwrap();

        let mut client_interface_configuration = HTTPClientInterfaceSetupDataNative(HTTPClientInterfaceSetupData {
                url: format!("http://{}", address),
            })
                .create_interface()
                .await
                .unwrap();

        // get client socket
        let client_socket = async_next_pin_box(&mut client_interface_configuration.new_sockets_iterator).await.unwrap().unwrap();

        // check client properties
        assert_eq!(
            client_interface_configuration.properties.name,
            Some(format!("http://{}", address)),
        );
        assert_eq!(
            client_interface_configuration.properties.channel,
            "http"
        );
        assert_eq!(
            client_interface_configuration.properties.interface_type,
            "http-client"
        );

        // check server properties
        assert_eq!(
            server_interface_configuration.properties.name,
            Some(address)
        );
        assert_eq!(
            server_interface_configuration.properties.channel,
            "http"
        );
        assert_eq!(
            server_interface_configuration.properties.interface_type,
            "http-server"
        );

        // send data from client to server and back
        let message = DXBBlock::new_with_body(b"request");
        let message_clone = message.clone();

        let response_message = DXBBlock::new_with_body(b"response");
        let response_message_clone = response_message.clone();

        join!(
            async move {
                // get tmp socket on server side
                let server_socket = async_next_pin_box(&mut server_interface_configuration.new_sockets_iterator).await.unwrap().unwrap();
                // receive data on server
                test_receive_block(&mut server_socket.iterator.unwrap(), message_clone).await;

                // send data from server to client (sent via HTTP response)
                test_send_block_sync_once_callback(&server_socket.send_callback, response_message_clone.clone());
            },
            async move {
                // sleep 100ms to wait for server to accept connections
                tokio::time::sleep(core::time::Duration::from_millis(100)).await;
                // send data from client to server
                test_send_block_async_callback(&client_socket.send_callback, message).await;

                // receive response data on client
                test_receive_block(&mut client_socket.iterator.unwrap(), response_message).await;
            }
        );
    }
}