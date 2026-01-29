use super::tcp_common::TCPServerInterfaceSetupData;
use crate::{
    core::net::AddrParseError,
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            error::ComInterfaceError,
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::{InterfaceDirection, ComInterfaceProperties},
            socket::ComInterfaceSocketUUID,
        },
    },
    stdlib::{collections::HashMap, net::SocketAddr, sync::Arc},
};
use core::{prelude::rust_2024::*, result::Result, time::Duration};
use std::io;
use log::{error, info, warn};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpListener,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};
use crate::global::dxb_block::DXBBlock;
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SendFailure, SocketConfiguration, SocketProperties};
use futures::lock::Mutex;

impl TCPServerInterfaceSetupData {
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
                                                SendFailure(block)
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

impl ComInterfaceAsyncFactory for TCPServerInterfaceSetupData {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
    }

    fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "tcp-server".to_string(),
            channel: "tcp".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use datex_macros::async_test;
    use std::assert_matches;

    use crate::{
        network::{
            com_hub::errors::ComInterfaceCreateError,
            com_interfaces::{
                default_com_interfaces::tcp::tcp_common::TCPServerInterfaceSetupData,
            },
        },
    };

    #[async_test]
    async fn test_construct() {
        const PORT: u16 = 5088;
        let interface_configuration =
            TCPServerInterfaceSetupData::new_with_port(PORT)
                .create_interface()
                .await
                .unwrap();

        assert_eq!(
            interface_configuration.properties.name,
            Some(format!("0.0.0.0:{}", PORT))
        );
    }

    #[async_test]
    async fn test_invalid_address() {
        assert_matches!(
            TCPServerInterfaceSetupData::new_with_host_and_port(
                "invalid-address".to_string(),
                5088
            )
            .create_interface()
            .await,
            Err(ComInterfaceCreateError::InvalidSetupData(_))
        );
    }
}
