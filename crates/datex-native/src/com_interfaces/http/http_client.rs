use datex::channel::mpsc::create_unbounded_channel;
use datex::derive_setup_data;
use datex::network::com_hub::errors::ComInterfaceCreateError;
use datex::network::com_hub::managers::com_interface_manager::ComInterfaceAsyncFactoryResult;
use datex::network::com_interfaces::com_interface::factory::{ComInterfaceAsyncFactory, ComInterfaceConfiguration, SendCallback, SendFailure, SocketConfiguration, SocketProperties};
use datex::network::com_interfaces::com_interface::properties::{ComInterfaceProperties, InterfaceDirection};
use datex::network::com_interfaces::default_setup_data::http::http_client::HTTPClientInterfaceSetupData;

derive_setup_data!(HTTPClientInterfaceSetupDataNative, HTTPClientInterfaceSetupData);

impl HTTPClientInterfaceSetupDataNative {
    async fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {

        let (response_sender, mut response_receiver) = create_unbounded_channel::<Vec<u8>>();

        Ok(ComInterfaceConfiguration::new_single_socket(
            ComInterfaceProperties {
                name: Some(self.url.clone()),
                ..Self::get_default_properties()
            },
            SocketConfiguration::new(
                SocketProperties::new(InterfaceDirection::InOut, 1),
                async gen move {
                    while let Some(response_data) = response_receiver.next().await {
                        yield Ok(response_data);
                    }
                },
                SendCallback::new_async(move |block| {
                    let url = self.url.clone();
                    let mut response_sender = response_sender.clone();
                    async move {
                        let client = reqwest::Client::new();
                        let response = client.post(&url)
                            .body(block.to_bytes())
                            .send()
                            .await
                            .map_err(|e| {
                                println!("HTTP request error: {:#?}", e);
                                SendFailure(Box::new(block.clone()))
                            })?;
                        let status = response.status();
                        let bytes = response.bytes().await
                            .map_err(|e| {
                                println!("HTTP response read error: {:#?}", e);
                                SendFailure(Box::new(block.clone()))
                            })?;
                        response_sender.start_send(bytes.to_vec())
                            .unwrap();

                        if status.is_success() {
                            Ok(())
                        } else {
                            println!("HTTP request failed with status: {}", status);
                            Err(SendFailure(Box::new(block)))
                        }
                    }
                })
            )
        ))
    }
}

impl ComInterfaceAsyncFactory for HTTPClientInterfaceSetupDataNative {
    fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
        Box::pin(self.create_interface())
    }

    fn get_default_properties() -> ComInterfaceProperties {
        HTTPClientInterfaceSetupData::get_default_properties()
    }
}
