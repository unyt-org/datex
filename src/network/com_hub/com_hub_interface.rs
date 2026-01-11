use crate::{
    network::{
        com_hub::managers::interface_manager::{
            AsyncComInterfaceImplementationFactoryFn, InterfaceManager,
            SyncComInterfaceImplementationFactoryFn,
        },
        com_interfaces::com_interface::socket::ComInterfaceSocketUUID,
    },
    stdlib::{cell::RefCell, rc::Rc, string::String},
    task::{UnboundedReceiver, spawn_with_panic_notify},
};
use core::{prelude::rust_2024::*, result::Result};

use crate::{
    network::{
        com_hub::{
            ComHub, ComHubError, InterfacePriority,
            errors::InterfaceCreateError,
        },
        com_interfaces::com_interface::{
            ComInterface, ComInterfaceEvent, ComInterfaceUUID,
        },
    },
    values::value_container::ValueContainer,
};
use crate::network::com_hub::errors::InterfaceAddError;

/// Interface management methods
impl ComHub {
    /// Registers a new sync interface factory for the given interface type
    pub fn register_sync_interface_factory(
        &self,
        interface_type: String,
        factory: SyncComInterfaceImplementationFactoryFn,
    ) {
        self.interface_manager
            .borrow_mut()
            .register_sync_interface_factory(interface_type, factory);
    }

    pub fn register_async_interface_factory(
        &self,
        interface_type: String,
        factory: AsyncComInterfaceImplementationFactoryFn,
    ) {
        self.interface_manager
            .borrow_mut()
            .register_async_interface_factory(interface_type, factory);
    }

    /// Adds a new interface to the ComHub
    fn init_interface_event_listeners(&self, interface: Rc<ComInterface>) {
        // handle socket events
        self.handle_interface_socket_events(interface.clone());
        // handle interface events
        self.handle_interface_events(interface);
    }

    /// Internal method to handle interface events
    fn handle_interface_events(&self, interface: Rc<ComInterface>) {
        let interface_event_receiver =
            interface.take_interface_event_receiver();
        let uuid = interface.uuid().clone();
        spawn_with_panic_notify(
            &self.async_context,
            handle_interface_events(
                uuid,
                interface_event_receiver,
                self.interface_manager.clone(),
            ),
        );
    }

    /// Returns the com interface for a given socket UUID
    /// The interface and socket must be registered in the ComHub,
    /// otherwise a panic will be triggered
    pub(crate) fn dyn_interface_for_socket_uuid(
        &self,
        socket_uuid: &ComInterfaceSocketUUID,
    ) -> Rc<ComInterface> {
        let socket_manager = self.socket_manager.borrow();
        let socket = socket_manager.get_socket_by_uuid(socket_uuid);
        self.interface_manager
            .borrow()
            .dyn_interface_by_uuid(&socket.interface_uuid)
    }

    /// Registers an existing com interface on the ComHub and sets up event handling
    pub fn register_com_interface(
        &self,
        com_interface: Rc<ComInterface>,
        priority: InterfacePriority,
    ) -> Result<(), InterfaceAddError> {
        self.interface_manager
            .borrow_mut()
            .add_interface(com_interface.clone(), priority)?;
        self.handle_interface_socket_events(com_interface);
        Ok(())
    }

    /// Creates a new interface of the given type with the provided setup data
    pub async fn create_interface(
        &self,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
    ) -> Result<Rc<ComInterface>, InterfaceCreateError> {
        let com_interface = self
            .interface_manager
            .borrow_mut()
            .create_and_add_interface(interface_type, setup_data, priority)
            .await?;
        self.init_interface_event_listeners(com_interface.clone());
        Ok(com_interface)
    }

    /// Creates a new interface of the given type with the provided setup data
    /// If the interface does not support sync initialization, an error is returned
    pub fn create_interface_sync(
        &self,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
    ) -> Result<Rc<ComInterface>, InterfaceCreateError> {
        let com_interface = self
            .interface_manager
            .borrow_mut()
            .create_and_add_interface_sync(
                interface_type,
                setup_data,
                priority,
            )?;
        self.init_interface_event_listeners(com_interface.clone());
        Ok(com_interface)
    }

    pub async fn remove_interface(
        &self,
        interface_uuid: ComInterfaceUUID,
    ) -> Result<(), ComHubError> {
        self.interface_manager
            .borrow_mut()
            .remove_interface(interface_uuid)
            .await
    }

    pub fn has_interface(&self, interface_uuid: &ComInterfaceUUID) -> bool {
        self.interface_manager
            .borrow()
            .has_interface(interface_uuid)
    }
}

async fn handle_interface_events(
    uuid: ComInterfaceUUID,
    mut receiver_queue: UnboundedReceiver<ComInterfaceEvent>,
    interface_manager: Rc<RefCell<InterfaceManager>>,
) {
    while let Some(event) = receiver_queue.next().await {
        interface_manager
            .borrow_mut()
            .handle_interface_event(&uuid, event);
    }
}
