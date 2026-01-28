use crate::{
    channel::mpsc::UnboundedReceiver,
    network::{
        com_hub::{
            ComHub, ComHubError, InterfacePriority,
            MAX_CONCURRENT_COM_INTERFACE_SOCKETS_EMBASSY,
            MAX_CONCURRENT_COM_INTERFACES_EMBASSY,
            errors::{ComInterfaceCreateError, InterfaceAddError},
            managers::interfaces_manager::{
                DynInterfaceImplementationFactoryFn, InterfacesManager,
            },
        },
        com_interfaces::com_interface::{
            ComInterfaceReceivers, ComInterfaceStateEvent, ComInterfaceUUID,
            ComInterfaceUtils, factory::ComInterfaceSyncFactory,
            socket::ComInterfaceSocketUUID,
        },
    },
    runtime::AsyncContext,
    stdlib::{
        cell::{Ref, RefCell},
        rc::Rc,
        string::String,
    },
    task::spawn_with_panic_notify,
    values::value_container::ValueContainer,
};
use core::{prelude::rust_2024::*, result::Result};
use datex_core::network::com_interfaces::com_interface::{
    ComInterfaceWithReceivers, factory::ComInterfaceAsyncFactory,
};

/// Interface management methods
impl ComHub {
    /// Registers a new sync interface factory for the given interface type
    pub fn register_sync_interface_factory<T: ComInterfaceSyncFactory>(&self) {
        self.interface_manager
            .borrow_mut()
            .register_sync_interface_factory::<T>();
    }

    pub fn register_async_interface_factory<T: ComInterfaceAsyncFactory>(
        &self,
    ) {
        self.interface_manager
            .borrow_mut()
            .register_async_interface_factory::<T>();
    }

    pub fn register_dyn_interface_factory(
        &self,
        interface_type: String,
        factory: DynInterfaceImplementationFactoryFn,
    ) {
        self.interface_manager
            .borrow_mut()
            .register_dyn_interface_factory(interface_type, factory);
    }

    /// Returns the com interface for a given socket UUID
    /// The interface and socket must be registered in the ComHub,
    /// otherwise a panic will be triggered
    pub(crate) fn dyn_interface_for_socket_uuid(
        &self,
        socket_uuid: &ComInterfaceSocketUUID,
    ) -> Ref<'_, ComInterfaceUtils> {
        let socket_manager = self.socket_manager.borrow();
        let socket = socket_manager.get_socket_by_uuid(socket_uuid);
        Ref::map(self.interface_manager.borrow(), |manager| {
            manager.get_interface_by_uuid(&socket.interface_uuid)
        })
    }

    /// Registers an existing com interface on the ComHub and sets up event handling
    pub fn _register_com_interface(
        &self,
        com_interface_with_receivers: ComInterfaceWithReceivers,
        priority: InterfacePriority,
    ) -> Result<(), InterfaceAddError> {
        let (com_interface, receivers) = com_interface_with_receivers;
        let uuid = com_interface.uuid().clone();
        self.interface_manager
            .borrow_mut()
            .add_interface(com_interface, priority)?;
        self.init_interface_event_listeners(
            self.interface_manager.borrow().get_interface_by_uuid(&uuid),
            receivers,
        );
        Ok(())
    }

    /// Creates a new interface of the given type with the provided setup data
    pub async fn create_interface(
        &self,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
    ) -> Result<ComInterfaceUUID, ComInterfaceCreateError> {
        let (com_interface_uuid, receivers) =
            InterfacesManager::create_and_add_interface(
                self.interface_manager.clone(),
                interface_type,
                setup_data,
                priority,
            )
            .await?;
        let created_sockets;
        {
            let interface_manager = self.interface_manager.borrow();
            let interface =
                interface_manager.get_interface_by_uuid(&com_interface_uuid);
            created_sockets = interface.properties().created_sockets.clone();

            // set up event listeners
            self.init_interface_event_listeners(interface, receivers);
        }

        // wait for all initially created sockets to be registered
        // if let Some(created_sockets) = created_sockets {
        //     info!("waiting for created sockets: {:?}", created_sockets);
        //     for socket_uuid in created_sockets {
        //         let receiver = self.socket_manager
        //             .borrow_mut()
        //             .get_socket_registration_waiter(&socket_uuid);
        //         receiver.await.expect("Socket registration waiter failed");
        //     }
        // }

        Ok(com_interface_uuid)
    }

    /// Creates a new interface of the given type with the provided setup data
    /// If the interface does not support sync initialization, an error is returned
    pub fn create_interface_sync(
        &self,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
    ) -> Result<ComInterfaceUUID, ComInterfaceCreateError> {
        let mut interface_manager = self.interface_manager.borrow_mut();
        let (com_interface_uuid, receivers) = interface_manager
            .create_and_add_interface_sync(
                interface_type,
                setup_data,
                priority,
            )?;
        let interface_manager = self.interface_manager.borrow();
        self.init_interface_event_listeners(
            interface_manager.get_interface_by_uuid(&com_interface_uuid),
            receivers,
        );
        Ok(com_interface_uuid)
    }

    pub fn remove_interface(
        &self,
        interface_uuid: ComInterfaceUUID,
    ) -> Result<(), ComHubError> {
        self.interface_manager
            .borrow_mut()
            .destroy_interface(&interface_uuid)?;

        self.socket_manager
            .borrow_mut()
            .remove_sockets_for_interface_uuid(&interface_uuid);

        Ok(())
    }

    pub fn has_interface(&self, interface_uuid: &ComInterfaceUUID) -> bool {
        self.interface_manager
            .borrow()
            .has_interface(interface_uuid)
    }
}

#[cfg_attr(feature = "embassy_runtime", embassy_executor::task(pool_size = MAX_CONCURRENT_COM_INTERFACES_EMBASSY))]
async fn handle_interface_events_task(
    uuid: ComInterfaceUUID,
    mut receiver_queue: UnboundedReceiver<ComInterfaceStateEvent>,
    interface_manager: Rc<RefCell<InterfacesManager>>,
) {
    while let Some(event) = receiver_queue.next().await {
        interface_manager
            .borrow_mut()
            .handle_interface_event(&uuid, event);
    }
}
