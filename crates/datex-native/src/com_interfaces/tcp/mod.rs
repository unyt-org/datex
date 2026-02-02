pub mod tcp_client;
pub mod tcp_server;


#[cfg(test)]
mod tests {
    use ntest_timeout::timeout;
    use tokio::join;
    use datex::network::com_interfaces::com_interface::factory::{ComInterfaceAsyncFactory};

    use crate::com_interfaces::tcp::tcp_client::TCPClientInterfaceSetupDataNative;
    use crate::com_interfaces::tcp::tcp_server::TCPServerInterfaceSetupDataNative;
    use crate::com_interfaces::tests::test_client_server_sockets;
    use datex::network::com_interfaces::default_setup_data::tcp::tcp_client::TCPClientInterfaceSetupData;
    use datex::network::com_interfaces::default_setup_data::tcp::tcp_server::TCPServerInterfaceSetupData;
    use datex::utils::async_iterators::async_next_pin_box;

    #[tokio::test]
    #[timeout(2000)]
    async fn test_connect_and_communicate() {
        
        const PORT: u16 = 12456;
        let address= format!("0.0.0.0:{}", PORT);

        let mut server_interface_configuration =
            TCPServerInterfaceSetupDataNative(TCPServerInterfaceSetupData::new_with_port(PORT))
                .create_interface()
                .await
                .unwrap();

        let (client_interface_configuration, server_socket) = join!(
            // create client interface connection
            TCPClientInterfaceSetupDataNative(TCPClientInterfaceSetupData {
                address: address.clone(),
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
            Some(address.clone())
        );
        assert_eq!(
            client_interface_configuration.properties.channel,
            "tcp"
        );
        assert_eq!(
            client_interface_configuration.properties.interface_type,
            "tcp-client"
        );

        // check server properties
        assert_eq!(
            server_interface_configuration.properties.name,
            Some(address)
        );
        assert_eq!(
            server_interface_configuration.properties.channel,
            "tcp"
        );
        assert_eq!(
            server_interface_configuration.properties.interface_type,
            "tcp-server"
        );

        test_client_server_sockets(
            server_socket,
            client_socket,
        ).await;
    }
}