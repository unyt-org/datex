use crate::network::com_interfaces::com_interface::{ComInterface, ComInterfaceImplEvent};

use crate::network::com_hub::errors::InterfaceCreateError;
use crate::network::com_interfaces::com_interface::implementation::{
    ComInterfaceImplementation, ComInterfaceSyncFactory,
};
use crate::network::com_interfaces::com_interface::properties::{
    InterfaceDirection, InterfaceProperties,
};
use crate::stdlib::rc::Rc;
use crate::stdlib::string::ToString;
use crate::task::{spawn_with_panic_notify_default, UnboundedReceiver, UnboundedSender};
use crate::values::core_values::endpoint::Endpoint;
use core::prelude::rust_2024::*;
use core::result::Result;
use core::time::Duration;

/// A simple local loopback interface that puts outgoing data
/// back into the incoming queue.
pub struct LocalLoopbackInterface;
impl ComInterfaceImplementation for LocalLoopbackInterface {}

impl LocalLoopbackInterface {
    /// background task to handle com hub events (e.g. outgoing messages)
    async fn event_handler_task(
        mut sender: UnboundedSender<Vec<u8>>,
        mut receiver: UnboundedReceiver<ComInterfaceImplEvent>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                ComInterfaceImplEvent::SendBlock(block, _) => {
                    sender.start_send(block).unwrap();
                }
                _ => {}
            }
        }
    }
}

impl ComInterfaceSyncFactory for LocalLoopbackInterface {
    type SetupData = ();

    fn create(
        _setup_data: Self::SetupData,
        com_interface: Rc<ComInterface>,
    ) -> Result<(Self, InterfaceProperties), InterfaceCreateError> {
        // directly create a socket and register it
        let (socket_uuid, sender) = com_interface
            .socket_manager()
            .lock()
            .unwrap()
            .create_and_init_socket(InterfaceDirection::InOut, 1);
        com_interface
            .socket_manager()
            .lock()
            .unwrap()
            .register_socket_with_endpoint(socket_uuid, Endpoint::LOCAL, 1)?;
        
        
        // TODO: use async context
        // spawn event handler task for impl events
        spawn_with_panic_notify_default(
            Self::event_handler_task(
                sender,
                com_interface.take_interface_impl_event_receiver(),
            )
        );

        Ok((
            LocalLoopbackInterface,
            Self::get_default_properties(),
        ))
    }

    fn get_default_properties() -> InterfaceProperties {
        InterfaceProperties {
            interface_type: "local".to_string(),
            channel: "local".to_string(),
            round_trip_time: Duration::from_millis(0),
            max_bandwidth: u32::MAX,
            ..InterfaceProperties::default()
        }
    }
}
