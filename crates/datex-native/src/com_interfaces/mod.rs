use datex::network::com_hub::ComHub;

pub mod tcp;
pub mod websocket;
pub mod http;
pub mod serial;
pub mod webrtc;


/// Registers all enabled native interface factories to the provided ComHub.
pub fn register_native_interface_factories(com_hub: &ComHub) {
    #[cfg(feature = "websocket")]
    {
        com_hub.register_async_interface_factory::<websocket::websocket_client::WebSocketClientInterfaceSetupDataNative>();
        com_hub.register_async_interface_factory::<websocket::websocket_server::WebSocketServerInterfaceSetupDataNative>();
    }
    #[cfg(feature = "serial")]
    {
        com_hub.register_async_interface_factory::<serial::serial_client::SerialClientInterfaceSetupDataNative>();
    }
    #[cfg(feature = "tcp")]
    {
        com_hub.register_async_interface_factory::<tcp::tcp_client::TCPClientInterfaceSetupDataNative>();
        com_hub.register_async_interface_factory::<tcp::tcp_server::TCPServerInterfaceSetupDataNative>();
    }
    #[cfg(feature = "http")]
    {
        com_hub.register_async_interface_factory::<http::http_server::HTTPServerInterfaceSetupDataNative>();
        com_hub.register_async_interface_factory::<http::http_client::HTTPClientInterfaceSetupDataNative>();
    }
    // TODO:
    // #[cfg(feature = "webrtc")]
}


#[cfg(test)]
pub mod tests {
    use datex::global::dxb_block::DXBBlock;
    use datex::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SocketConfiguration, SocketDataIterator};
    use datex::utils::async_iterators::async_next_pin_box;

    /// Test utility function to test client-server communication for two sockets
    /// Sends and receives data in both directions
    pub async fn test_client_server_sockets(
        server_socket_configuration: SocketConfiguration,
        client_socket_configuration: SocketConfiguration,
    ) {
        // send data from client to server
        let message = DXBBlock::new_with_body(b"Hello, World!");
        test_send_block_async_callback(&client_socket_configuration.send_callback, message.clone()).await;

        // receive data on server
        test_receive_block(&mut server_socket_configuration.iterator.unwrap(), message).await;

        // send data from server to client
        let response_message = DXBBlock::new_with_body(b"Hello back!");
        test_send_block_async_callback(&server_socket_configuration.send_callback, response_message.clone()).await;

        // receive data on client
        test_receive_block(&mut client_socket_configuration.iterator.unwrap(), response_message).await;
    }
    
    /// Test utility function to test client-server communication for two interfaces
    pub async fn test_client_server_interfaces(
        server_interface_configuration: ComInterfaceConfiguration,
        client_interface_configuration: ComInterfaceConfiguration,
    ) {
        // check if sockets were created
        let mut server_socket_iterator = server_interface_configuration.new_sockets_iterator;
        let server_socket = async_next_pin_box(&mut server_socket_iterator).await;
        assert!(server_socket.is_some());
        let server_socket = server_socket.unwrap();
        assert!(server_socket.is_ok());
        let server_socket = server_socket.unwrap();

        let mut client_socket_iterator = client_interface_configuration.new_sockets_iterator;
        let client_socket = async_next_pin_box(&mut client_socket_iterator).await;
        assert!(client_socket.is_some());
        let client_socket = client_socket.unwrap();
        assert!(client_socket.is_ok());
        let client_socket = client_socket.unwrap();

        test_client_server_sockets(
            server_socket,
            client_socket
        ).await;
    }

    /// Test utility function to send a block using the provided send callback.
    /// Asserts that the callback is asynchronous and successfully sends the block.
    pub(crate) async fn test_send_block_async_callback(send_callback: &Option<SendCallback>, block: DXBBlock) {
        match send_callback {
            Some(SendCallback::Async(callback)) => {
                callback.call(block).await.unwrap();
            }
            _ => panic!("Expected async send callback"),
        }
    }

    /// Test utility function to send a block using the provided send callback.
    /// Asserts that the callback is asynchronous and successfully sends the block.
    pub(crate) fn test_send_block_sync_once_callback(send_callback: &Option<SendCallback>, block: DXBBlock) {
        match send_callback {
            Some(SendCallback::SyncOnce(callback)) => {
                callback(block).unwrap();
            }
            _ => panic!("Expected sync once callback"),
        }
    }
    
    /// Test utility function to receive a block from the provided socket data iterator.
    /// Asserts that the received block matches the provided block.
    pub(crate) async fn test_receive_block(block_iterator: &mut SocketDataIterator, matches_block: DXBBlock) {
        let received_block = async_next_pin_box(block_iterator).await;
        assert!(received_block.is_some());
        let received_block = received_block.unwrap();
        assert!(received_block.is_ok());
        let received_data = received_block.unwrap();
        assert_eq!(received_data, matches_block.to_bytes());
    }
}