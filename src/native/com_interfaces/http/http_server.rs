use crate::{derive_setup_data, stdlib::{net::SocketAddr, rc::Rc, sync::Arc}};
use core::time::Duration;
use async_tiny::{Response, Server};

use crate::network::com_interfaces::default_setup_data::http::http_server::{HTTPServerInterfaceSetupData};
use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceAsyncFactoryResult,
            },
            properties::ComInterfaceProperties,
        },
    },
};
use log::{debug};
use url::Url;
use datex_core::network::com_interfaces::com_interface::properties::InterfaceDirection;
use crate::global::dxb_block::DXBBlock;
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SendFailure, SendSuccess, SocketConfiguration, SocketProperties};

derive_setup_data!(HTTPServerInterfaceSetupDataNative, HTTPServerInterfaceSetupData);

impl HTTPServerInterfaceSetupDataNative {
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

impl ComInterfaceAsyncFactory for HTTPServerInterfaceSetupDataNative {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
    }

    fn get_default_properties() -> ComInterfaceProperties {
        HTTPServerInterfaceSetupData::get_default_properties()
    }
}
