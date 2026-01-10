use super::tcp_common::{TCPError, TCPServerInterfaceSetupData};
use crate::network::com_hub::errors::InterfaceCreateError;
use crate::network::com_interfaces::com_interface::ComInterface;
use crate::network::com_interfaces::com_interface::error::ComInterfaceError;
use crate::network::com_interfaces::com_interface::implementation::{
    ComInterfaceAsyncFactory, ComInterfaceImplementation,
    ComInterfaceSyncFactory,
};
use crate::network::com_interfaces::com_interface::properties::{
    InterfaceDirection, InterfaceProperties,
};
use crate::network::com_interfaces::com_interface::socket::{
    ComInterfaceSocket, ComInterfaceSocketUUID,
};
use crate::std_sync::Mutex;
use crate::stdlib::cell::RefCell;
use crate::stdlib::collections::HashMap;
use crate::stdlib::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use crate::stdlib::pin::Pin;
use crate::stdlib::rc::Rc;
use crate::stdlib::sync::Arc;
use crate::task::UnboundedSender;
use crate::task::spawn_with_panic_notify_default;
use core::future::Future;
use core::prelude::rust_2024::*;
use core::result::Result;
use core::time::Duration;
use datex_macros::{com_interface, create_opener};
use log::{error, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub struct TCPServerNativeInterface {
    pub address: SocketAddr,
    com_interface: Rc<ComInterface>,
    pub tx_by_socket:
        Arc<Mutex<HashMap<ComInterfaceSocketUUID, Arc<Mutex<OwnedWriteHalf>>>>>,
}

impl TCPServerNativeInterface {
    async fn create(
        setup_data: TCPServerInterfaceSetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
        let address = SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            setup_data.port,
        ));

        info!("Spinning up server at {address}");

        let listener =
            TcpListener::bind(address.clone()).await.map_err(|e| {
                InterfaceCreateError::InterfaceError(
                    ComInterfaceError::connection_error_with_details(e),
                )
            })?;
        info!("Server listening on {address}");

        let tx_by_socket = Arc::new(Mutex::new(HashMap::new()));
        let tx_by_socket_clone = tx_by_socket.clone();

        // TODO #615: use normal spawn (thread)? currently leads to global context panic
        let manager = com_interface.socket_manager();
        spawn_with_panic_notify_default(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let (socket_uuid, sender) =
                            manager.lock().unwrap().create_and_init_socket(
                                InterfaceDirection::InOut,
                                1,
                            );
                        let (read_half, write_half) = stream.into_split();
                        tx_by_socket_clone.try_lock().unwrap().insert(
                            socket_uuid,
                            Arc::new(Mutex::new(write_half)),
                        );

                        spawn_with_panic_notify_default(async move {
                            Self::handle_client(read_half, sender).await
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {e}");
                        continue;
                    }
                }
            }
        });

        Ok((
            TCPServerNativeInterface {
                address,
                com_interface,
                tx_by_socket,
            },
            Self::get_default_properties(),
        ))
    }

    async fn handle_client(
        mut rx: OwnedReadHalf,
        mut bytes_in_sender: UnboundedSender<Vec<u8>>,
    ) {
        let mut buffer = [0u8; 1024];
        loop {
            match rx.read(&mut buffer).await {
                Ok(0) => {
                    warn!("Connection closed by peer");
                    break;
                }
                Ok(n) => {
                    bytes_in_sender.start_send(buffer[..n].to_vec()).expect(
                        "Failed to send received data to ComInterfaceSocket",
                    );
                }
                Err(e) => {
                    error!("Failed to read from socket: {e}");
                    break;
                }
            }
        }
    }
}

impl ComInterfaceAsyncFactory for TCPServerNativeInterface {
    type SetupData = TCPServerInterfaceSetupData;
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
            TCPServerNativeInterface::create(setup_data, com_interface)
                .await
                .map_err(|e| {
                    InterfaceCreateError::InterfaceError(
                        ComInterfaceError::connection_error_with_details(
                            format!("{e:?}"),
                        ),
                    )
                })
        })
    }

    fn get_default_properties() -> InterfaceProperties {
        InterfaceProperties {
            interface_type: "tcp-server".to_string(),
            channel: "tcp".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            ..InterfaceProperties::default()
        }
    }
}

impl ComInterfaceImplementation for TCPServerNativeInterface {
    fn send_block<'a>(
        &'a self,
        block: &'a [u8],
        socket: ComInterfaceSocketUUID,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        let tx_queues = self.tx_by_socket.clone();
        let tx_queues = tx_queues.try_lock().unwrap();
        let tx = tx_queues.get(&socket);
        if tx.is_none() {
            error!("Client is not connected");
            return Box::pin(async { false });
        }
        let tx = tx.unwrap().clone();
        Box::pin(
            async move { tx.try_lock().unwrap().write(block).await.is_ok() },
        )
    }
    fn handle_destroy<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        todo!("#207")
    }
    fn handle_reconnect<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = bool> + 'a>> {
        todo!()
    }
}
