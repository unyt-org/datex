use datex::{derive_setup_data, compat::{net::SocketAddr, sync::Arc}};
use core::{
     result::Result, str::FromStr,
};
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures::lock::Mutex;
use datex::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::{InterfaceDirection, ComInterfaceProperties},
        },
    },
};
use datex::global::dxb_block::DXBBlock;
use datex::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SendFailure, SocketProperties, SocketConfiguration};
use datex::network::com_interfaces::default_setup_data::websocket::websocket_server::WebSocketServerInterfaceSetupData;

derive_setup_data!(WebSocketServerInterfaceSetupDataNative, WebSocketServerInterfaceSetupData);


impl WebSocketServerInterfaceSetupDataNative {
    async fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let addr = SocketAddr::from_str(&self.bind_address)
            .map_err(ComInterfaceCreateError::invalid_setup_data)?;

        let listener = TcpListener::bind(&addr).await.map_err(|err| {
            ComInterfaceCreateError::connection_error_with_details(err)
        })?;

        info!("WebSocket Server listening on {addr}");

        Ok(ComInterfaceConfiguration::new(
            ComInterfaceProperties {
                name: Some(addr.to_string()),
                connectable_interfaces: WebSocketServerInterfaceSetupData::get_clients_setup_data(self.0.accept_addresses)?,
                ..Self::get_default_properties()
            },
            async gen move {
                loop {
                    // get next websocket connection
                    match Self::get_next_websocket_connection(&listener).await {
                        Ok((mut read, write)) => {
                            info!("Accepted new WebSocket connection");
                            // yield new socket data
                            yield Ok(SocketConfiguration::new(
                                SocketProperties::new(InterfaceDirection::InOut, 1),
                                // socket incoming blocks iterator
                                async gen move {
                                    // read blocks
                                    loop {
                                        match read.next().await {
                                            Some(Ok(Message::Binary(bin))) => {
                                                yield Ok(bin);
                                            }
                                            Some(Ok(_)) => {
                                                error!("Invalid message type received");
                                                return yield Err(());
                                            }
                                            Some(Err(e)) => {
                                                error!("WebSocket error from {addr}: {e}");
                                                return yield Err(())
                                            }
                                            None => {
                                                // Connection closed by peer
                                                return;
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
                                            .send(Message::Binary(block.to_bytes())).await
                                            .map_err(|e| {
                                                error!("WebSocket write error: {e}");
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
            }
        ))
    }

    async fn get_next_websocket_connection(listener: &TcpListener) -> Result<
        (SplitStream<WebSocketStream<TcpStream>>, Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>),
        ()
    > {
        // new sockets iterators are yielded on client connection
        let next_socket = listener.accept().await;
        match next_socket {
            Ok((stream, addr)) => {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        let (write, read) = ws_stream.split();
                        let write = Arc::new(Mutex::new(write));
                        Ok((read, write))
                    }
                    Err(e) => {
                        error!("WebSocket handshake failed with {addr}: {e}");
                        Err(())
                    }
                }
            }
            Err(e) => {
                error!("Failed to accept connection: {e}");
                Err(())
            }
        }
    }
}

impl ComInterfaceAsyncFactory for WebSocketServerInterfaceSetupDataNative {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
    }

    fn get_default_properties() -> ComInterfaceProperties {
        WebSocketServerInterfaceSetupData::get_default_properties()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use datex_core::native_global_context::init_global_context_native;
    use crate::{
        network::{
            com_hub::errors::ComInterfaceCreateError,
        },
    };
    use super::*;

    #[tokio::test]
    async fn test_construct() {
        
        let address = "0.0.0.0:1234".to_string();

        let interface_configuration =
            WebSocketServerInterfaceSetupDataNative(WebSocketServerInterfaceSetupData {
                bind_address: address.clone(),
                accept_addresses: None,
            })
                .create_interface()
                .await
                .unwrap();

        assert_eq!(
            interface_configuration.properties.name,
            Some(address)
        );
    }

    #[tokio::test]
    async fn test_construct_invalid_address() {
        
        assert_matches!(
            WebSocketServerInterfaceSetupDataNative(WebSocketServerInterfaceSetupData {
                bind_address: "1.2.3".to_string(),
                accept_addresses: None,
            })
            .create_interface()
            .await,
            Err(ComInterfaceCreateError::InvalidSetupData(_))
        );
    }
}
