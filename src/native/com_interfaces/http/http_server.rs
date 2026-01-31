use crate::{derive_setup_data, stdlib::{net::SocketAddr}};
use core::str::FromStr;
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
use datex_core::network::com_interfaces::com_interface::properties::InterfaceDirection;
use crate::global::dxb_block::DXBBlock;
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SendFailure, SendSuccess, SocketConfiguration, SocketProperties};

derive_setup_data!(HTTPServerInterfaceSetupDataNative, HTTPServerInterfaceSetupData);

impl HTTPServerInterfaceSetupDataNative {
    async fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let addr = SocketAddr::from_str(&self.bind_address)
            .map_err(ComInterfaceCreateError::invalid_setup_data)?;

        let mut server = Server::http(&addr.to_string(), false).await.map_err(|e| {
            ComInterfaceCreateError::connection_error_with_details(e)
        })?;

        println!("HTTP server running on http://{addr}");

        Ok(ComInterfaceConfiguration::new(
            ComInterfaceProperties {
                name: Some(addr.to_string()),
                connectable_interfaces: HTTPServerInterfaceSetupData::get_clients_setup_data(self.0.accept_addresses)?,
                ..Self::get_default_properties()
            },
            async gen move {
                // create new tmp socket for each new incoming request
                while let Some(request) = server.next().await {
                    println!("Accepted new HTTP request: {} {}", request.method(), request.url());
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
                                    println!("HTTP response send error: {:#?}", e);
                                    SendFailure(Box::new(block))
                                })
                                .map(|_| {
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
