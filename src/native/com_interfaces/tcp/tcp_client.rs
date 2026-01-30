use crate::network::com_interfaces::default_setup_data::tcp::tcp_client::TCPClientInterfaceSetupData;

use crate::{derive_setup_data, network::{
    com_hub::errors::ComInterfaceCreateError,
    com_interfaces::com_interface::{
        error::ComInterfaceError,
        factory::{
            ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
        },
        properties::{InterfaceDirection, ComInterfaceProperties},
    },
}, stdlib::{net::SocketAddr, sync::Arc}};
use core::{
    prelude::rust_2024::*, result::Result, str::FromStr, time::Duration,
};
use futures_util::lock::Mutex;
use log::{error, warn};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream},
    select,
};
use datex_core::network::com_interfaces::com_interface::factory::ComInterfaceConfiguration;
use crate::network::com_interfaces::com_interface::factory::{SendCallback, SendFailure, SocketConfiguration, SocketProperties};


derive_setup_data!(TCPClientInterfaceSetupDataNative, TCPClientInterfaceSetupData);


/// Implementation of the TCP Client Native Interface
impl TCPClientInterfaceSetupDataNative {
    async fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let address = SocketAddr::from_str(&self.address)
            .map_err(ComInterfaceCreateError::invalid_setup_data)?;

        let stream = TcpStream::connect(address).await.map_err(|error| {
            ComInterfaceCreateError::connection_error_with_details(error)
        })?;

        let (mut read, write) = stream.into_split();
        let write = Arc::new(Mutex::new(write));

        Ok(ComInterfaceConfiguration::new_single_socket(
            ComInterfaceProperties {
                name: Some(self.0.address),
                ..Self::get_default_properties()
            },
            SocketConfiguration::new(
                SocketProperties::new(
                    InterfaceDirection::InOut,
                    1,
                ),
                async gen move {
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
                                return yield Err(())
                            }
                        }
                    }
                },
                SendCallback::new_async(move |block| {
                    let write = write.clone();
                    async move {
                        write
                            .lock()
                            .await
                            .write_all(&block.to_bytes()).await
                            .map_err(|e| {
                                error!("WebSocket write error: {e}");
                                SendFailure(Box::new(block))
                            })
                    }
                }),
            ),
        ))
    }
}

impl ComInterfaceAsyncFactory for TCPClientInterfaceSetupDataNative {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
    }

    fn get_default_properties() -> ComInterfaceProperties {
        TCPClientInterfaceSetupData::get_default_properties()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::com_interfaces::default_setup_data::tcp::tcp_client::TCPClientInterfaceSetupData;
    use crate::native_global_context::init_global_context_native;

    #[tokio::test]
    async fn test_construct_invalid_address() {
        init_global_context_native();
        const ADDRESS: &str = "1.2.3";
        let result = TCPClientInterfaceSetupDataNative(TCPClientInterfaceSetupData {
            address: ADDRESS.to_string(),
        })
            .create_interface()
            .await;
        assert!(matches!(result, Err(ComInterfaceCreateError::InvalidSetupData(_))));
    }
}