use futures::lock::Mutex;

use crate::{
    channel::mpsc::create_unbounded_channel,
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::properties::{
            ComInterfaceProperties, InterfaceDirection,
        },
    },
    values::core_values::endpoint::Endpoint,
};

use crate::{
    global::dxb_block::DXBBlock,
    network::com_interfaces::com_interface::factory::{
        ComInterfaceConfiguration, SendCallback, SendSuccess,
        SocketConfiguration, SocketProperties,
    },
    prelude::*,
};
use core::time::Duration;

/// A simple local loopback interface that puts outgoing data
/// back into the incoming queue.
pub struct LocalLoopbackInterfaceSetupData;

impl LocalLoopbackInterfaceSetupData {
    pub(crate) fn create_interface(
        self,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let (tx, mut rx) = create_unbounded_channel::<Vec<u8>>();
        let tx = Arc::new(Mutex::new(tx));
        Ok(ComInterfaceConfiguration::new_single_socket(
            Self::get_default_properties(),
            SocketConfiguration::new(
                SocketProperties::new_with_direct_endpoint(
                    InterfaceDirection::InOut,
                    1,
                    Endpoint::LOCAL.clone(),
                ),
                async gen move {
                    loop {
                        let data = rx.next().await;
                        if let Some(data) = data {
                            yield Ok(data);
                        } else {
                            break;
                        }
                    }
                },
                SendCallback::new_sync(move |block: DXBBlock| {
                    let data = block.to_bytes();
                    tx.try_lock().unwrap().start_send(data).expect(
                        "Failed to send data to local loopback interface",
                    );
                    Ok(SendSuccess::Sent)
                }),
            ),
        ))
    }

    fn get_default_properties() -> ComInterfaceProperties {
        ComInterfaceProperties {
            interface_type: "local".to_string(),
            channel: "local".to_string(),
            auto_identify: false,
            round_trip_time: Duration::from_millis(0),
            max_bandwidth: u32::MAX,
            ..ComInterfaceProperties::default()
        }
    }
}
