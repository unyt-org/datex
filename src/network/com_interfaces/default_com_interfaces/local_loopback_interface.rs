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
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SocketProperties, SocketConfiguration};

/// A simple local loopback interface that puts outgoing data
/// back into the incoming queue.
#[derive(Deserialize)]
pub struct LocalLoopbackInterfaceSetupData;

impl ComInterfaceSyncFactory for LocalLoopbackInterfaceSetupData {
    fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        Ok(
            ComInterfaceConfiguration::new_single_socket(
                Self::get_default_properties(),
                SocketConfiguration::new_out(
                    SocketProperties::new_with_endpoint(
                        InterfaceDirection::InOut,
                        1,
                        Endpoint::LOCAL.clone()
                    ),
                    SendCallback::new_sync(
                        move |block: DXBBlock| {
                            Ok(SendSuccess::SentWithNewIncomingData(vec![block.to_bytes()]))
                        }
                    )
                )
            )
        )
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