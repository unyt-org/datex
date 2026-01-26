use crate::{
    network::com_interfaces::com_interface::properties::InterfaceDirection,
    stdlib::rc::Rc, stdlib::string::String, stdlib::boxed::Box, stdlib::string::ToString,
};
use core::{cell::RefCell, pin::Pin};
use log::info;

use crate::{
    collections::HashMap,
    network::{
        com_hub::{
            ComHubError, InterfacePriority,
            errors::{InterfaceAddError, ComInterfaceCreateError},
        },
        com_interfaces::com_interface::{
            ComInterface, ComInterfaceProxy, ComInterfaceReceivers,
            ComInterfaceStateEvent, ComInterfaceUUID,
            factory::{ComInterfaceAsyncFactory, ComInterfaceSyncFactory},
            properties::InterfaceProperties,
        },
    },
    runtime::AsyncContext,
    values::value_container::ValueContainer,
};
use crate::network::com_interfaces::com_interface::factory::ComInterfaceConfiguration;

type InterfaceMap =
    HashMap<ComInterfaceUUID, (ComInterface, InterfacePriority)>;

pub type SyncComInterfaceImplementationFactoryFn =
    fn(
        setup_data: ValueContainer,
        proxy: ComInterfaceProxy,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError>;

pub type ComInterfaceAsyncFactoryResult = Pin<
    Box<
        dyn Future<Output = Result<ComInterfaceConfiguration, ComInterfaceCreateError>>
            + 'static,
    >,
>;

pub type AsyncComInterfaceImplementationFactoryFn =
    fn(
        setup_data: ValueContainer,
        proxy: ComInterfaceProxy,
    ) -> ComInterfaceAsyncFactoryResult;

pub type DynInterfaceImplementationFactoryFn = Rc<
    dyn Fn(ValueContainer, ComInterfaceProxy) -> ComInterfaceAsyncFactoryResult,
>;

#[derive(Clone)]
pub enum SyncOrAsyncComInterfaceImplementationFactoryFn {
    Sync(SyncComInterfaceImplementationFactoryFn),
    Async(AsyncComInterfaceImplementationFactoryFn),
    Dyn(DynInterfaceImplementationFactoryFn),
}

#[derive(Default)]
pub struct InterfacesManager {
    /// a list of all available interface factories, keyed by their interface type
    pub interface_factories:
        HashMap<String, SyncOrAsyncComInterfaceImplementationFactoryFn>,

    /// a list of all available interfaces, keyed by their UUID
    pub interfaces: InterfaceMap,
}

/// Manages the registered interfaces and their factories
/// Allows creating, adding, removing and querying interfaces
/// Also handles interface events (lifecycle management)
impl InterfacesManager {
    /// Registers a new sync interface factory for a specific interface implementation.
    /// This allows the ComHub to create new instances of the interface on demand.
    pub fn register_sync_interface_factory<T: ComInterfaceSyncFactory>(
        &mut self,
    ) {
        let interface_type = T::get_default_properties().interface_type;
        self.interface_factories.insert(
            interface_type,
            SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(T::factory),
        );
    }

    /// Registers a new async interface factory for a specific interface implementation.
    /// This allows the ComHub to create new instances of the interface on demand.
    pub fn register_async_interface_factory<T: ComInterfaceAsyncFactory>(
        &mut self,
    ) {
        let interface_type = T::get_default_properties().interface_type;
        self.interface_factories.insert(
            interface_type,
            SyncOrAsyncComInterfaceImplementationFactoryFn::Async(T::factory),
        );
    }

    /// Registers a new custom async interface factory for a specific interface type.
    /// This allows the ComHub to create new instances of the interface on demand.
    pub fn register_dyn_interface_factory(
        &mut self,
        interface_type: String,
        factory: DynInterfaceImplementationFactoryFn,
    ) {
        self.interface_factories.insert(
            interface_type,
            SyncOrAsyncComInterfaceImplementationFactoryFn::Dyn(factory),
        );
    }

    /// Creates a new interface instance using the registered factory
    /// for the specified interface type if it exists.
    /// The interface is opened and added to the ComHub.
    pub async fn create_and_add_interface(
        self_rc: Rc<RefCell<Self>>,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
        async_context: AsyncContext,
    ) -> Result<(ComInterfaceUUID, ComInterfaceReceivers), ComInterfaceCreateError>
    {
        info!("creating interface {interface_type}");
        let factory = self_rc
            .borrow()
            .interface_factories
            .get(interface_type)
            .cloned();
        if let Some(factory) = factory {
            match factory {
                SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(_) => {
                    self_rc.borrow_mut().create_and_add_interface_sync(
                        interface_type,
                        setup_data,
                        priority,
                        async_context,
                    )
                }
                SyncOrAsyncComInterfaceImplementationFactoryFn::Async(_)
                | SyncOrAsyncComInterfaceImplementationFactoryFn::Dyn(_) => {
                    let (interface, receivers) =
                        ComInterface::create_from_async_factory_fn(
                            &factory,
                            setup_data,
                            async_context,
                        )
                        .await?;
                    self_rc
                        .borrow_mut()
                        .add_interface(interface, priority)
                        .map_err(|e| e.into())
                        .map(|interface| (interface.uuid.clone(), receivers))
                }
            }
        } else {
            Err(ComInterfaceCreateError::InterfaceTypeNotRegistered(
                interface_type.to_string(),
            ))
        }
    }

    /// Creates a new interface instance using the registered sync factory
    /// for the specified interface type if it exists.
    /// If the factory is async, an error is returned.
    /// The interface is opened and added to the ComHub.
    pub fn create_and_add_interface_sync(
        &mut self,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
        async_context: AsyncContext,
    ) -> Result<(ComInterfaceUUID, ComInterfaceReceivers), ComInterfaceCreateError>
    {
        info!("creating interface sync {interface_type}");
        if let Some(factory) = self.interface_factories.get(interface_type) {
            match factory {
                SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(
                    sync_factory,
                ) => {
                    let (interface, receivers) =
                        ComInterface::create_from_sync_factory_fn(
                            sync_factory,
                            setup_data,
                            async_context,
                        )?;
                    self.add_interface(interface, priority)
                        .map_err(|e| e.into())
                        .map(|interface| (interface.uuid.clone(), receivers))
                }
                SyncOrAsyncComInterfaceImplementationFactoryFn::Async(_)
                | SyncOrAsyncComInterfaceImplementationFactoryFn::Dyn(_) => Err(
                    ComInterfaceCreateError::InterfaceCreationRequiresAsyncContext,
                ),
            }
        } else {
            Err(ComInterfaceCreateError::InterfaceTypeNotRegistered(
                interface_type.to_string(),
            ))
        }
    }

    /// Checks if the interface with the given UUID exists in the manager
    pub fn has_interface(&self, interface_uuid: &ComInterfaceUUID) -> bool {
        self.interfaces.contains_key(interface_uuid)
    }

    /// Returns the com interface for a given UUID
    /// The interface is returned as a dynamic trait object
    pub fn try_interface_by_uuid(
        &self,
        uuid: &ComInterfaceUUID,
    ) -> Option<&ComInterface> {
        self.interfaces.get(uuid).map(|(interface, _)| interface)
    }

    /// Returns the com interface for a given UUID
    /// The interface must be registered in the ComHub,
    /// otherwise a panic will be triggered
    pub fn get_interface_by_uuid(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> &ComInterface {
        self.try_interface_by_uuid(interface_uuid)
            .unwrap_or_else(|| {
                core::panic!("Interface for uuid {interface_uuid} not found")
            })
    }

    /// Adds an interface to the manager, checking for duplicates
    pub fn add_interface(
        &mut self,
        interface: ComInterface,
        priority: InterfacePriority,
    ) -> Result<&ComInterface, InterfaceAddError> {
        let uuid = interface.uuid().clone();
        if self.interfaces.contains_key(&uuid) {
            return Err(InterfaceAddError::InterfaceAlreadyExists);
        }

        // make sure the interface can send if a priority is set
        if priority != InterfacePriority::None
            && interface.properties().direction == InterfaceDirection::In
        {
            return Err(
                InterfaceAddError::InvalidInterfaceDirectionForFallbackInterface,
            );
        }

        self.interfaces.insert(uuid.clone(), (interface, priority));
        Ok(self.get_interface_by_uuid(&uuid))
    }

    /// Returns the priority of the interface with the given UUID
    pub fn interface_priority(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Option<InterfacePriority> {
        self.interfaces
            .get(interface_uuid)
            .map(|(_, priority)| *priority)
    }

    /// User can proactively remove an interface from the hub.
    /// This will destroy the interface and it's sockets (perform deep cleanup)
    pub fn destroy_interface(
        &mut self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Result<(), ComHubError> {
        info!("Removing interface {interface_uuid}");
        let interface = &mut self
            .interfaces
            .get_mut(interface_uuid)
            .ok_or(ComHubError::InterfaceDoesNotExist)?
            .0;
        {
            // Async close the interface (stop tasks, server, cleanup internal data)
            interface.destroy();
            // TODO: await until closed asynchronously?
        }

        self.cleanup_interface(interface_uuid)?;

        Ok(())
    }

    /// The internal cleanup function that removes the interface from the hub
    /// and disabled the default interface if it was set to this interface
    fn cleanup_interface(
        &mut self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Result<(), ComHubError> {
        self.interfaces
            .remove(interface_uuid)
            .ok_or(ComHubError::InterfaceDoesNotExist)
            .map(|_| ())
    }

    /// Handles interface events received from interfaces
    pub fn handle_interface_event(
        &mut self,
        interface_uuid: &ComInterfaceUUID,
        event: ComInterfaceStateEvent,
    ) {
        if let ComInterfaceStateEvent::Destroyed = event {
            // FIXME should probably do more cleanup here, but this was what com hub did before
            // try cleanup, already cleaned up when destroyed via destroy_interface, so the call may fail
            // but we can ignore that
            let _ = self.cleanup_interface(interface_uuid);
        }
    }
}
