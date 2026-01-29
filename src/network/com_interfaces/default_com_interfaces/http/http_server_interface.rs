use bytes::Bytes;
use core::cell::RefCell;

use crate::{
    channel::mpsc::UnboundedReceiver,
    collections::HashMap,
    stdlib::{net::SocketAddr, rc::Rc, sync::Arc},
};
use core::time::Duration;
use async_tiny::{Response, Server};
use futures::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use super::http_common::{HTTPServerInterfaceSetupData};
use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::ComInterfaceProperties,
            socket::ComInterfaceSocketUUID,
        },
    },
};
use log::{debug, error, info};
use tokio::sync::{RwLock, broadcast, mpsc};
use url::Url;
use datex_core::network::com_interfaces::com_interface::properties::InterfaceDirection;
use crate::global::dxb_block::DXBBlock;
use crate::network::com_interfaces::com_interface::error::ComInterfaceError;
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SendFailure, SendSuccess, SocketConfiguration, SocketProperties};

impl HTTPServerInterfaceSetupData {
    async fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let address: String = format!("http://0.0.0.0:{}", self.port);
        let address = Url::parse(&address)
            .map_err(ComInterfaceCreateError::invalid_setup_data)?;

        let addr: SocketAddr = address
            .socket_addrs(|| None)
            .map_err(ComInterfaceCreateError::invalid_setup_data)?
            .first()
            .cloned()
            .ok_or(ComInterfaceCreateError::invalid_setup_data(
                "Socket address invalid",
            ))?;

        println!("HTTP server starting on http://{addr}");
        let mut server = Server::http(&addr.to_string(), false).await.map_err(|e| {
            ComInterfaceCreateError::connection_error_with_details(e)
        })?;
        
        Ok(ComInterfaceConfiguration::new(
            Self::get_default_properties(),
            async gen move {
                // create new tmp socket for each new incoming request
                while let Some(request) = server.next().await {
                    let request_body = request.body().to_vec();
                    // yield new socket data
                    yield Ok(SocketConfiguration::new(
                        SocketProperties::new(InterfaceDirection::InOut, 1),
                        // handle request data
                        async gen move {
                            yield Ok(request_body);
                        },
                        // socket send callback (single send per request)
                        SendCallback::new_sync_once(move |block: DXBBlock| {
                            let response = Response::from_data(block.to_bytes());
                            request.respond(response)
                                .map_err(|e| {
                                    SendFailure(block)
                                })
                                .map(|_| {
                                    debug!("HTTP response sent successfully");
                                    SendSuccess::Sent
                                })
                        })
                    ));

                }
            }
        ))
    }
}

impl ComInterfaceAsyncFactory for HTTPServerInterfaceSetupData {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
    }

    fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "http-server".to_string(),
            channel: "http".to_string(),
            round_trip_time: Duration::from_millis(20),
            max_bandwidth: 1000,
            direction: InterfaceDirection::InOut,
            ..ComInterfaceProperties::default()
        }
    }
}
