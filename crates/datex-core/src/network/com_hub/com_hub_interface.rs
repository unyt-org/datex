use crate::{
    network::{
        com_hub::{
            ComHub, ComHubError, InterfacePriority,
            errors::{ComInterfaceCreateError, InterfaceAddError},
            managers::com_interface_manager::DynInterfaceImplementationFactoryFn,
        },
        com_interfaces::com_interface::{
            ComInterfaceUUID,
            factory::{ComInterfaceConfiguration, ComInterfaceSyncFactory},
            socket::ComInterfaceSocketUUID,
        },
    },
    values::value_container::ValueContainer,
};

use crate::{
    network::com_interfaces::com_interface::{
        factory::ComInterfaceAsyncFactory, properties::ComInterfaceProperties,
    },
    prelude::*,
};

/// Interface management methods
impl ComHub {
    /// Registers a new sync interface factory for the given interface type
    pub fn register_sync_interface_factory<T: ComInterfaceSyncFactory>(&self) {
        self.interfaces_manager
            .register_sync_interface_factory::<T>();
    }

    pub fn register_async_interface_factory<T: ComInterfaceAsyncFactory>(
        &self,
    ) {
        self.interfaces_manager
            .register_async_interface_factory::<T>();
    }

    pub fn register_dyn_interface_factory(
        &self,
        interface_type: String,
        factory: DynInterfaceImplementationFactoryFn,
    ) {
        self.interfaces_manager
            .register_dyn_interface_factory(interface_type, factory);
    }

    /// Returns the com interface for a given socket UUID
    /// The interface and socket must be registered in the ComHub,
    /// otherwise a panic will be triggered
    pub(crate) fn dyn_interface_for_socket_uuid(
        &self,
        socket_uuid: &ComInterfaceSocketUUID,
    ) -> Rc<ComInterfaceProperties> {
        let socket = self.socket_manager.get_socket_by_uuid(socket_uuid);
        self.interfaces_manager
            .get_interface_by_uuid(&socket.interface_uuid)
    }

    /// Adds a new interface to the ComHub based on the provided configuration
    pub fn add_interface_from_configuration(
        self: Rc<Self>,
        interface_configuration: ComInterfaceConfiguration,
        priority: InterfacePriority,
    ) -> Result<(), InterfaceAddError> {
        let uuid = interface_configuration.uuid();
        self.interfaces_manager.add_interface(
            uuid,
            interface_configuration.properties.clone(),
            priority,
        )?;
        self.register_com_interface_handler(interface_configuration, priority);
        Ok(())
    }

    /// Creates a new interface of the given type with the provided setup data
    pub async fn create_interface(
        self: Rc<Self>,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
    ) -> Result<ComInterfaceUUID, ComInterfaceCreateError> {
        let interface_configuration = self
            .interfaces_manager
            .create_and_add_interface(interface_type, setup_data, priority)
            .await?;

        let uuid = interface_configuration.uuid();
        // add event handler task
        self.register_com_interface_handler(interface_configuration, priority);

        Ok(uuid)
    }

    /// Creates a new interface of the given type with the provided setup data
    /// If the interface does not support sync initialization, an error is returned
    pub fn create_interface_sync(
        self: Rc<Self>,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
    ) -> Result<ComInterfaceUUID, ComInterfaceCreateError> {
        let interface_configuration =
            self.interfaces_manager.create_and_add_interface_sync(
                interface_type,
                setup_data,
                priority,
            )?;

        let uuid = interface_configuration.uuid();
        // add event handler task
        self.register_com_interface_handler(interface_configuration, priority);

        Ok(uuid)
    }

    pub fn remove_interface(
        &self,
        interface_uuid: ComInterfaceUUID,
    ) -> Result<(), ComHubError> {
        self.interfaces_manager.destroy_interface(&interface_uuid)?;

        self.socket_manager
            .remove_sockets_for_interface_uuid(&interface_uuid);

        Ok(())
    }

    pub fn has_interface(&self, interface_uuid: &ComInterfaceUUID) -> bool {
        self.interfaces_manager.has_interface(interface_uuid)
    }
}


#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec::Vec;
    use core::future::join;
    use tokio::task::{LocalSet};
    use crate::channel::mpsc::{create_unbounded_channel, UnboundedSender};
    use crate::global::dxb_block::{DXBBlock, IncomingSection};
    use crate::global::protocol_structures::routing_header::SignatureType;
    use crate::network::com_hub::{ComHub, InterfacePriority};
    use crate::network::com_hub::metadata::ComHubMetadata;
    use crate::network::com_hub::test_utils::{get_coupled_com_hubs, run_with_coupled_com_hubs};
    use crate::network::com_interfaces::com_interface::ComInterfaceUUID;
    use crate::network::com_interfaces::com_interface::factory::{ComInterfaceConfiguration, SendCallback, SendSuccess, SocketConfiguration, SocketProperties};
    use crate::network::com_interfaces::com_interface::properties::{ComInterfaceProperties, InterfaceDirection};
    use crate::task::{timeout};
    use crate::values::core_values::endpoint::Endpoint;

    fn get_metadata_sockets(
        com_hub_metadata: ComHubMetadata,
    ) -> Vec<(Option<Endpoint>, Option<i8>)> {
        com_hub_metadata
            .interfaces
            .into_iter()
            .flat_map(|e| {
                e.sockets
                    .into_iter()
                    .map(|s| (s.endpoint, s.properties.map(|p| p.distance)))
            })
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_add_interface_from_configuration() {
        let configuration = ComInterfaceConfiguration::new_single_socket(
            ComInterfaceProperties::default(),
            SocketConfiguration::new_in_out(
                SocketProperties::new(InterfaceDirection::InOut, 1),
                async gen move {},
                SendCallback::new_sync(|_block| {
                    Ok(SendSuccess::Sent)
                })
            )
        );
        let uuid = configuration.uuid();
        let properties = configuration.properties.clone();

        let (incoming_sections_sender, _incoming_sections_receiver) =
            create_unbounded_channel::<IncomingSection>();
        let (com_hub, _task_future) = ComHub::create(Endpoint::default(), incoming_sections_sender);

        // add interface
        com_hub.clone().add_interface_from_configuration(configuration, InterfacePriority::default()).unwrap();
        assert_eq!(com_hub.interfaces_manager.interfaces.borrow().get(&uuid).unwrap().properties, properties);

        // remove interface
        com_hub.remove_interface(uuid.clone()).unwrap();
        assert!(!com_hub.has_interface(&uuid));
    }

    #[test]
    fn test_remove_nonexistent_interface() {
        let (incoming_sections_sender, _incoming_sections_receiver) =
            create_unbounded_channel::<IncomingSection>();
        let (com_hub, _task_future) = ComHub::create(Endpoint::default(), incoming_sections_sender);

        let result = com_hub.remove_interface(ComInterfaceUUID::new());
        assert!(result.is_err());
    }


    #[tokio::test]
    #[cfg(any(feature = "std", feature = "allow_unsigned_blocks"))]
    async fn test_connected_interfaces() {
        let (peer_a, peer_b) = get_coupled_com_hubs();

        // run task futures for 10ms to allow sockets to connect
        let _ = timeout(core::time::Duration::from_millis(10), join!(
            peer_a.task_future,
            peer_b.task_future
        )).await;

        let sockets_a = get_metadata_sockets(peer_a.com_hub.get_metadata());
        let sockets_b = get_metadata_sockets(peer_b.com_hub.get_metadata());

        // check that each peer has exactly one socket, and that they are correctly connected to each other
        assert_eq!(sockets_a.len(), 1);
        assert_eq!(sockets_b.len(), 1);

        assert_eq!(sockets_a[0].0.as_ref().unwrap().to_string(), "@test-b");
        assert_eq!(sockets_b[0].0.as_ref().unwrap().to_string(), "@test-a");
    }

    #[tokio::test]
    #[cfg(feature = "std")]
    async fn test_interfaces_send_block() {
        flexi_logger::init();

        run_with_coupled_com_hubs(|peer_a, mut peer_b| async move {
            // create block to send from A to B
            let block_a_to_b_body = [1, 2, 3];
            let mut block_a_to_b = DXBBlock::new_with_body(&block_a_to_b_body);
            block_a_to_b.set_receivers(vec![peer_b.com_hub.endpoint.clone()]);
            block_a_to_b.routing_header.flags.set_signature_type(SignatureType::Unencrypted);

            // send block from A to B
            peer_a.com_hub.send_own_block_async(block_a_to_b).await.unwrap();

            // receive block on B
            let section = peer_b.incoming_sections_receiver.next().await.unwrap();
            match section {
                IncomingSection::SingleBlock((Some(block), _)) => {
                    assert_eq!(block.body, block_a_to_b_body);
                },
                _ => panic!("Expected block section")
            }
        }).await;
    }
}