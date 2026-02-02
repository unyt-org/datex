use datex::network::com_interfaces::default_setup_data::tcp::tcp_server::TCPServerInterfaceSetupData;
use core::net::AddrParseError;
use datex::{derive_setup_data, network::{
    com_hub::errors::ComInterfaceCreateError,
    com_interfaces::com_interface::{
        factory::{
            ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
        },
        properties::{InterfaceDirection, ComInterfaceProperties},
    },
}};
use core::{ result::Result};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use log::{error, info, warn};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpListener,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};
use datex::global::dxb_block::DXBBlock;
use datex::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SendFailure, SocketConfiguration, SocketProperties};
use futures::lock::Mutex;

derive_setup_data!(TCPServerInterfaceSetupDataNative, TCPServerInterfaceSetupData);

impl TCPServerInterfaceSetupDataNative {
    async fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let host = self.host.clone().unwrap_or_else(|| "0.0.0.0".to_string());

        let address: SocketAddr = format!("{}:{}", host, self.port)
            .parse()
            .map_err(|e: AddrParseError| {
                ComInterfaceCreateError::InvalidSetupData(e.to_string())
            })?;

        let listener = TcpListener::bind(address).await.map_err(|e| {
            ComInterfaceCreateError::connection_error_with_details(e)
        })?;
        info!("TCP Server listening on {address}");

        Ok(ComInterfaceConfiguration::new(
            ComInterfaceProperties {
                name: Some(format!("{}:{}", host, self.port)),
                ..Self::get_default_properties()
            },
            async gen move {
                loop {
                    // get next websocket connection
                    match Self::get_next_socket_connection(&listener).await {
                        Ok((addr, mut read, write)) => {
                            info!("Accepted new TCP connection from {addr}");
                            // yield new socket data
                            yield Ok(SocketConfiguration::new(
                                SocketProperties::new(InterfaceDirection::InOut, 1),
                                // socket incoming blocks iterator
                                async gen move {
                                    // read blocks
                                    loop {
                                        let mut buffer = [0u8; 1024];
                                        match read.read(&mut buffer).await {
                                            Ok(0) => {
                                                warn!("Connection closed by peer");
                                                return;
                                            }
                                            Ok(n) => {
                                                yield Ok(buffer[..n].to_vec());
                                            }
                                            Err(e) => {
                                                error!("Failed to read from socket: {e}");
                                                return yield Err(());
                                            }
                                        }
                                    }
                                },
                                // socket send callback
                                SendCallback::new_async(move |block: DXBBlock| {
                                    let write = write.clone();
                                    async move {
                                        write
                                            .lock()
                                            .await
                                            .write_all(&block.to_bytes())
                                            .await
                                            .map_err(|e| {
                                                error!("TCP write error: {e}");
                                                SendFailure(Box::new(block))
                                            })
                                    }
                                })
                            ));
                        }
                        Err(_) => {
                            // Failed to accept connection, continue to next
                            continue;
                        }
                    }
                }
            },
        ))
    }

    async fn get_next_socket_connection(listener: &TcpListener) -> Result<(SocketAddr, OwnedReadHalf, Arc<Mutex<OwnedWriteHalf>>), io::Error> {
        let (stream, addr) = listener.accept().await?;
        // Handle the client connection
        let (tcp_read_half, tcp_write_half) = stream.into_split();
        Ok((addr, tcp_read_half, Arc::new(Mutex::new(tcp_write_half))))
    }
}

impl ComInterfaceAsyncFactory for TCPServerInterfaceSetupDataNative {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
    }

    fn get_default_properties() -> ComInterfaceProperties {
        TCPServerInterfaceSetupData::get_default_properties()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use datex::{
        network::{
            com_hub::errors::ComInterfaceCreateError,
        },
    };
    use super::*;

    #[tokio::test]
    async fn test_construct() {
        
        const PORT: u16 = 5088;
        let interface_configuration =
            TCPServerInterfaceSetupDataNative(TCPServerInterfaceSetupData::new_with_port(PORT))
                .create_interface()
                .await
                .unwrap();

        assert_eq!(
            interface_configuration.properties.name,
            Some(format!("0.0.0.0:{}", PORT))
        );
    }

    #[tokio::test]
    async fn test_construct_invalid_address() {
        
        assert_matches!(
            TCPServerInterfaceSetupDataNative(TCPServerInterfaceSetupData::new_with_host_and_port(
                "invalid-address".to_string(),
                5088
            ))
            .create_interface()
            .await,
            Err(ComInterfaceCreateError::InvalidSetupData(_))
        );
    }
}
