use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            factory::ComInterfaceSyncFactory,
            properties::{InterfaceDirection, InterfaceProperties},
        },
    },
    stdlib::{string::ToString, vec, vec::Vec},
    values::core_values::endpoint::Endpoint,
};
use core::{prelude::rust_2024::*, result::Result, time::Duration};
use serde::Deserialize;
use datex_core::network::com_interfaces::com_interface::factory::{SendCallback, SendSuccess};
use crate::global::dxb_block::DXBBlock;
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, NewSocketsIterator, SocketConfiguration, SocketDataIterator};
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;

/// A simple local loopback interface that puts outgoing data
/// back into the incoming queue.
#[derive(Deserialize)]
pub struct LocalLoopbackInterfaceSetupData;

impl ComInterfaceSyncFactory for LocalLoopbackInterfaceSetupData {
    fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        Ok(ComInterfaceConfiguration {
            properties: Self::get_default_properties(),
            send_callback: SendCallback::new_sync(
                move |(block, _uuid): (DXBBlock, ComInterfaceSocketUUID)| {
                    Ok(SendSuccess::SentAndReceivedData(vec![block.to_bytes()]))
                }
            ),
            close_callback: None,
            new_sockets_iterator: NewSocketsIterator::new_single(SocketDataIterator::from(
                SocketConfiguration::new_with_endpoint(
                    InterfaceDirection::InOut,
                    1,
                    Endpoint::LOCAL.clone()
                )
            )),
        })
    }

    fn get_default_properties() -> InterfaceProperties {
        InterfaceProperties {
            interface_type: "local".to_string(),
            channel: "local".to_string(),
            auto_identify: false,
            round_trip_time: Duration::from_millis(0),
            max_bandwidth: u32::MAX,
            ..InterfaceProperties::default()
        }
    }
}