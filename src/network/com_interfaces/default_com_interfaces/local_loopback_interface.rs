use crate::{
    network::{
        com_hub::errors::InterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterfaceEvent,
            factory::ComInterfaceSyncFactory,
            properties::{InterfaceDirection, InterfaceProperties},
        },
    },
    stdlib::{string::ToString, vec, vec::Vec},
    task::spawn_with_panic_notify,
    values::core_values::endpoint::Endpoint,
};
use core::{prelude::rust_2024::*, result::Result, time::Duration};
use datex_core::network::com_interfaces::com_interface::ComInterfaceProxy;
use serde::Deserialize;
use crate::channel::mpsc::UnboundedSender;

/// A simple local loopback interface that puts outgoing data
/// back into the incoming queue.
#[derive(Deserialize)]
pub struct LocalLoopbackInterfaceSetupData;

impl ComInterfaceSyncFactory for LocalLoopbackInterfaceSetupData {
    fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> Result<InterfaceProperties, InterfaceCreateError> {
        // directly create a socket and register it
        let (socket_uuid, sender) = com_interface_proxy
            .create_and_init_socket_with_direct_endpoint(
                InterfaceDirection::InOut,
                1,
                Endpoint::LOCAL.clone(),
            );

        let async_context = com_interface_proxy.async_context.clone();

        // spawn event handler task for impl events
        spawn_with_panic_notify(
            &async_context,
            local_loopback_task(
                com_interface_proxy,
                sender,
            ),
        );

        Ok(InterfaceProperties {
            created_sockets: Some(vec![socket_uuid]),
            ..Self::get_default_properties()
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

#[cfg_attr(feature = "embassy_runtime", embassy_executor::task)]
async fn local_loopback_task(
    mut com_interface_proxy: ComInterfaceProxy,
    mut sender: UnboundedSender<Vec<u8>>,
) {
    while let Some(event) =
        com_interface_proxy.event_receiver.next().await
    {
        if let ComInterfaceEvent::SendBlock(block, _) = event {
            sender.start_send(block.to_bytes()).unwrap();
        }
    }
}