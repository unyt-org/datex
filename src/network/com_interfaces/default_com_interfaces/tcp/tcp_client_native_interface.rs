use super::tcp_common::TCPClientInterfaceSetupData;

use crate::{
    channel::{
        futures_intrusive::ManualResetEvent,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            error::ComInterfaceError,
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::{InterfaceDirection, ComInterfaceProperties},
            state::{ComInterfaceState, ComInterfaceStateWrapper},
        },
    },
    stdlib::{net::SocketAddr, sync::Arc},
    task::spawn_with_panic_notify_default,
};
use core::{
    prelude::rust_2024::*, result::Result, str::FromStr, time::Duration,
};
use futures_util::lock::Mutex;
use log::{error, warn};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp::OwnedWriteHalf},
    select,
};
use tungstenite::Message;
use datex_core::network::com_interfaces::com_interface::factory::ComInterfaceConfiguration;
use crate::network::com_interfaces::com_interface::factory::{SendCallback, SendFailure, SocketConfiguration, SocketProperties};

/// Implementation of the TCP Client Native Interface
impl TCPClientInterfaceSetupData {
    async fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let address = SocketAddr::from_str(&self.address)
            .map_err(ComInterfaceCreateError::invalid_setup_data)?;

        let stream = TcpStream::connect(address).await.map_err(|error| {
            ComInterfaceError::connection_error_with_details(error)
        })?;

        let (mut read, write) = stream.into_split();
        let write = Arc::new(Mutex::new(write));
        
        Ok(ComInterfaceConfiguration::new_single_socket(
            ComInterfaceProperties {
                name: Some(self.address),
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
                                SendFailure(block)
                            })
                    }
                }),
            ),
        ))
    }
}

impl ComInterfaceAsyncFactory for TCPClientInterfaceSetupData {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
    }

    fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "tcp-client".to_string(),
            channel: "tcp".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            ..ComInterfaceProperties::default()
        }
    }
}
