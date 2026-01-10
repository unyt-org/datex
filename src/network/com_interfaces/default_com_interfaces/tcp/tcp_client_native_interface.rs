use super::tcp_common::TCPClientInterfaceSetupData;

use crate::network::com_hub::errors::InterfaceCreateError;
use crate::network::com_interfaces::com_interface::error::ComInterfaceError;
use crate::network::com_interfaces::com_interface::implementation::{
    ComInterfaceAsyncFactory, ComInterfaceImplementation,
    ComInterfaceSyncFactory,
};
use crate::network::com_interfaces::com_interface::properties::{
    InterfaceDirection, InterfaceProperties,
};
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;
use crate::network::com_interfaces::com_interface::state::ComInterfaceState;
use crate::network::com_interfaces::com_interface::ComInterface;
use crate::stdlib::net::SocketAddr;
use crate::stdlib::pin::Pin;
use crate::stdlib::rc::Rc;
use crate::stdlib::sync::Arc;
use crate::task::spawn;
use core::cell::RefCell;
use core::future::Future;
use core::prelude::rust_2024::*;
use core::result::Result;
use core::str::FromStr;
use core::time::Duration;
use log::{error, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::select;
use tokio::sync::Notify;

pub struct TCPClientNativeInterface {
    pub address: SocketAddr,
    pub socket_uuid: ComInterfaceSocketUUID,
    tx: RefCell<OwnedWriteHalf>,
    com_interface: Rc<ComInterface>,
    shutdown_signal: Arc<Notify>,
}

impl TCPClientNativeInterface {
    async fn create(
        setup_data: TCPClientInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
        let address = SocketAddr::from_str(&setup_data.address)
            .map_err(InterfaceCreateError::invalid_setup_data)?;

        let stream = TcpStream::connect(address).await.map_err(|error| {
            ComInterfaceError::connection_error_with_details(error)
        })?;

        let (read_half, tx) = stream.into_split();

        let (socket_uuid, mut sender) = com_interface
            .socket_manager()
            .lock()
            .unwrap()
            .create_and_init_socket(InterfaceDirection::InOut, 1);

        let state = com_interface.state();
        let shutdown_signal = Arc::new(Notify::new());
        let shutdown_signal_clone = shutdown_signal.clone();

        spawn(async move {
            let mut reader = read_half;
            let mut buffer = [0u8; 1024];
            loop {
                select! {
                    next = reader.read(&mut buffer) => {
                        match next {
                            Ok(0) => {
                                warn!("Connection closed by peer");
                                state.lock().unwrap().set(ComInterfaceState::Destroyed);
                                break;
                            }
                            Ok(n) => {
                                sender.start_send(buffer[..n].to_vec()).unwrap();
                            }
                            Err(e) => {
                                error!("Failed to read from socket: {e}");
                                state
                                    .try_lock()
                                    .unwrap()
                                    .set(ComInterfaceState::Destroyed);
                                break;
                            }
                        }
                    }
                    _ = shutdown_signal_clone.notified() => {
                        break;
                    }
                }
            }
        });

        Ok((
            TCPClientNativeInterface {
                address,
                socket_uuid,
                tx: RefCell::new(tx),
                com_interface,
                shutdown_signal,
            },
            InterfaceProperties {
                name: Some(setup_data.address),
                ..Self::get_default_properties()
            },
        ))
    }
}

impl ComInterfaceImplementation for TCPClientNativeInterface {
    fn send_block<'a>(
        &'a self,
        block: &'a [u8],
        _: ComInterfaceSocketUUID,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        Box::pin(async move {
            match self.tx.borrow_mut().write_all(block).await {
                Ok(_) => true,
                Err(e) => {
                    error!("Failed to send data: {}", e);
                    false
                }
            }
        })
    }
    fn handle_destroy<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        todo!()
    }

    fn handle_reconnect<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        todo!()
    }
}

impl ComInterfaceAsyncFactory for TCPClientNativeInterface {
    type SetupData = TCPClientInterfaceSetupData;

    fn create(
        setup_data: Self::SetupData,
        com_interface: Rc<ComInterface>,
    ) -> Pin<
        Box<
            dyn Future<
                Output = Result<
                    (Self, InterfaceProperties),
                    InterfaceCreateError,
                >,
            >,
        >,
    > {
        Box::pin(async move {
            TCPClientNativeInterface::create(setup_data, com_interface).await
        })
    }

    fn get_default_properties() -> InterfaceProperties {
        InterfaceProperties {
            interface_type: "tcp-client".to_string(),
            channel: "tcp".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            ..InterfaceProperties::default()
        }
    }
}
