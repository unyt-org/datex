use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            properties::{InterfaceDirection, ComInterfaceProperties},
        },
    },
    stdlib::{string::ToString, vec, vec::Vec},
    values::core_values::endpoint::Endpoint,
};
use core::{prelude::rust_2024::*, result::Result, time::Duration};
use datex_core::network::com_interfaces::com_interface::factory::{SendCallback, SendSuccess};
use crate::global::dxb_block::DXBBlock;
use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SocketProperties, SocketConfiguration};
use crate::runtime::{Runtime};

/// A simple local loopback interface that puts outgoing data
/// back into the incoming queue.
pub struct LocalLoopbackInterfaceSetupData {
    pub(crate) runtime: Runtime
}

impl LocalLoopbackInterfaceSetupData {
    pub(crate) fn create_interface(self) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        Ok(
            ComInterfaceConfiguration::new_single_socket(
                Self::get_default_properties(),
                SocketConfiguration::new_out(
                    SocketProperties::new_with_direct_endpoint(
                        InterfaceDirection::InOut,
                        1,
                        Endpoint::LOCAL.clone()
                    ),
                    SendCallback::new_sync(
                        move |block: DXBBlock| {
                            // TODO: call runtime receive (sync) here
                            Ok(SendSuccess::SentWithNewIncomingData(block.to_bytes()))
                        }
                    )
                )
            )
        )
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