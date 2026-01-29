use crate::{
    network::{
        com_hub::{
            ComHub, ComHubError, InterfacePriority,
            errors::{ComInterfaceCreateError, InterfaceAddError},
            managers::com_interface_manager::{
                DynInterfaceImplementationFactoryFn,
            },
        },
        com_interfaces::com_interface::{
            ComInterfaceUUID,
            factory::ComInterfaceSyncFactory,
            socket::ComInterfaceSocketUUID,
        },
    },
    stdlib::{
        string::String,
    },
    values::value_container::ValueContainer,
};
use core::{prelude::rust_2024::*, result::Result};
use core::cell::Ref;
use datex_core::network::com_interfaces::com_interface::{
    factory::ComInterfaceAsyncFactory,
};
use crate::network::com_interfaces::com_interface::factory::ComInterfaceConfiguration;

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
    ) -> Ref<ComInterfaceConfiguration> {
        let socket = self.socket_manager.get_socket_by_uuid(socket_uuid);
        self.interfaces_manager.get_interface_by_uuid(&socket.interface_uuid)
    }

    /// Registers an existing com interface on the ComHub and sets up event handling
    pub fn _register_com_interface(
        &self,
        com_interface_configuration: ComInterfaceConfiguration,
        priority: InterfacePriority,
    ) -> Result<(), InterfaceAddError> {
        let uuid = com_interface_configuration.uuid().clone();
        self.interfaces_manager
            .add_interface(com_interface_configuration, priority)?;
        Ok(())
    }

    /// Creates a new interface of the given type with the provided setup data
    pub async fn create_interface(
        &self,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
    ) -> Result<ComInterfaceUUID, ComInterfaceCreateError> {
        let com_interface_uuid =
            self.interfaces_manager.create_and_add_interface(
                interface_type,
                setup_data,
                priority,
            )
            .await?;
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
        let com_interface_uuid = self.interfaces_manager
            .create_and_add_interface_sync(
                interface_type,
                setup_data,
                priority,
            )?;
        Ok(com_interface_uuid)
    }

    pub fn remove_interface(
        &self,
        interface_uuid: ComInterfaceUUID,
    ) -> Result<(), ComHubError> {
        self.interfaces_manager
            .destroy_interface(&interface_uuid)?;

        self.socket_manager
            .remove_sockets_for_interface_uuid(&interface_uuid);

        Ok(())
    }

    pub fn has_interface(&self, interface_uuid: &ComInterfaceUUID) -> bool {
        self.interfaces_manager
            .has_interface(interface_uuid)
    }
}