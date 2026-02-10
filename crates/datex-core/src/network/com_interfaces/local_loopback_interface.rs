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

pub use crate::std_sync::Mutex;
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

#[cfg(test)]
mod tests {
    use crate::{
        global::dxb_block::DXBBlock,
        network::com_interfaces::{
            com_interface::factory::SendCallback,
            local_loopback_interface::LocalLoopbackInterfaceSetupData,
        },
        utils::async_iterators::async_next_pin_box,
    };

    #[tokio::test]
    async fn test_local_loopback_interface() {
        let mut interface_configuration =
            LocalLoopbackInterfaceSetupData.create_interface().unwrap();
        assert_eq!(interface_configuration.properties.interface_type, "local");

        let socket = async_next_pin_box(
            &mut interface_configuration.new_sockets_iterator,
        )
        .await
        .unwrap()
        .unwrap();

        let block = DXBBlock::new_with_body(&[1, 2, 3]);
        let block_bytes = block.to_bytes();
        match socket.send_callback.unwrap() {
            SendCallback::Sync(callback) => {
                callback(block).unwrap();
            }
            _ => panic!("Expected sync send callback"),
        }

        let received_data = async_next_pin_box(&mut socket.iterator.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(block_bytes, received_data);
    }
}
