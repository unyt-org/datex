pub mod websocket_client;
pub mod websocket_server;

// TODO: move to integration tests once this is a separate crate
#[cfg(test)]
mod tests {
    use ntest_timeout::timeout;
    use tokio::join;
    use crate::native::com_interfaces::tests::{test_client_server_interfaces, test_client_server_sockets};
    use crate::native_global_context::init_global_context_native;
    use crate::network::com_interfaces::com_interface::factory::{ComInterfaceAsyncFactory};

    use crate::network::com_interfaces::default_setup_data::websocket::websocket_server::WebSocketServerInterfaceSetupData;
    use crate::native::com_interfaces::websocket::websocket_server::WebSocketServerInterfaceSetupDataNative;
    use crate::network::com_interfaces::default_setup_data::websocket::websocket_client::WebSocketClientInterfaceSetupData;
    use crate::native::com_interfaces::websocket::websocket_client::WebSocketClientInterfaceSetupDataNative;
    use crate::utils::async_iterators::async_next_pin_box;

    #[tokio::test]
    #[timeout(2000)]
    async fn test_connect_and_communicate() {
        
        let address= "0.0.0.0:45678".to_string();

        let mut server_interface_configuration =
            WebSocketServerInterfaceSetupDataNative(WebSocketServerInterfaceSetupData {
                bind_address: address.clone(),
                accept_addresses: None,
            })
                .create_interface()
                .await
                .unwrap();

        let (client_interface_configuration, server_socket) = join!(
            // create client interface connection
            WebSocketClientInterfaceSetupDataNative(WebSocketClientInterfaceSetupData {
                url: format!("ws://{}", address),
            })
                .create_interface(),
            // await connections on server side
            async_next_pin_box(&mut server_interface_configuration.new_sockets_iterator)
        );

        // get sockets
        let mut client_interface_configuration = client_interface_configuration.unwrap();
        let server_socket = server_socket.unwrap().unwrap();
        let client_socket = async_next_pin_box(&mut client_interface_configuration.new_sockets_iterator).await.unwrap().unwrap();

        // check client properties
        assert_eq!(
            client_interface_configuration.properties.name,
            Some(format!("ws://{}", address)),
        );
        assert_eq!(
            client_interface_configuration.properties.channel,
            "websocket"
        );
        assert_eq!(
            client_interface_configuration.properties.interface_type,
            "websocket-client"
        );

        // check server properties
        assert_eq!(
            server_interface_configuration.properties.name,
            Some(address)
        );
        assert_eq!(
            server_interface_configuration.properties.channel,
            "websocket"
        );
        assert_eq!(
            server_interface_configuration.properties.interface_type,
            "websocket-server"
        );

        test_client_server_sockets(
            server_socket,
            client_socket,
        ).await;
    }
}