use crate::{
    network::com_interfaces::com_interface::{
        implementation::ComInterfaceImpl, properties::InterfaceDirection,
    },
    stdlib::rc::Rc,
};
use core::pin::Pin;

use log::info;

use crate::{
    collections::HashMap,
    network::{
        com_hub::{
            ComHubError, InterfacePriority, errors::InterfaceCreateError,
        },
        com_interfaces::com_interface::{
            ComInterface, ComInterfaceEvent, ComInterfaceUUID,
            properties::InterfaceProperties,
        },
    },
    values::value_container::ValueContainer,
};
use crate::network::com_hub::errors::InterfaceAddError;

type InterfaceMap =
    HashMap<ComInterfaceUUID, (Rc<ComInterface>, InterfacePriority)>;

pub type SyncComInterfaceImplementationFactoryFn = fn(
    setup_data: ValueContainer,
    interface: Rc<ComInterface>,
) -> Result<
    (Box<dyn ComInterfaceImpl>, InterfaceProperties),
    InterfaceCreateError,
>;

pub type AsyncComInterfaceImplementationFactoryFn = fn(
    setup_data: ValueContainer,
    interface: Rc<ComInterface>,
) -> Pin<
    Box<
        dyn Future<
                Output = Result<
                    (Box<dyn ComInterfaceImpl>, InterfaceProperties),
                    InterfaceCreateError,
                >,
            > + 'static,
    >,
>;

pub enum SyncOrAsyncComInterfaceImplementationFactoryFn {
    Sync(SyncComInterfaceImplementationFactoryFn),
    Async(AsyncComInterfaceImplementationFactoryFn),
}

#[derive(Default)]
pub struct InterfaceManager {
    /// a list of all available interface factories, keyed by their interface type
    pub interface_factories:
        HashMap<String, SyncOrAsyncComInterfaceImplementationFactoryFn>,

    /// a list of all available interfaces, keyed by their UUID
    pub interfaces: InterfaceMap,
}

/// Manages the registered interfaces and their factories
/// Allows creating, adding, removing and querying interfaces
/// Also handles interface events (lifecycle management)
impl InterfaceManager {
    /// Registers a new sync interface factory for a specific interface implementation.
    /// This allows the ComHub to create new instances of the interface on demand.
    pub fn register_sync_interface_factory(
        &mut self,
        interface_type: String,
        factory: SyncComInterfaceImplementationFactoryFn,
    ) {
        self.interface_factories.insert(
            interface_type,
            SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(factory),
        );
    }

    /// Registers a new async interface factory for a specific interface implementation.
    /// This allows the ComHub to create new instances of the interface on demand.
    pub fn register_async_interface_factory(
        &mut self,
        interface_type: String,
        factory: AsyncComInterfaceImplementationFactoryFn,
    ) {
        self.interface_factories.insert(
            interface_type,
            SyncOrAsyncComInterfaceImplementationFactoryFn::Async(factory),
        );
    }

    /// Creates a new interface instance using the registered factory
    /// for the specified interface type if it exists.
    /// The interface is opened and added to the ComHub.
    pub async fn create_and_add_interface(
        &mut self,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
    ) -> Result<Rc<ComInterface>, InterfaceCreateError> {
        info!("creating interface {interface_type}");
        if let Some(factory) = self.interface_factories.get(interface_type) {
            match factory {
                SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(
                    sync_factory,
                ) => {
                    let interface = ComInterface::create_from_sync_factory_fn(
                        *sync_factory,
                        setup_data,
                    )?;
                    self.add_interface(interface.clone(), priority)
                        .map(|_| interface)
                        .map_err(|e| e.into())
                }
                SyncOrAsyncComInterfaceImplementationFactoryFn::Async(
                    async_factory,
                ) => {
                    let interface = ComInterface::create_from_async_factory_fn(
                        *async_factory,
                        setup_data,
                    )
                    .await?;

                    self.add_interface(interface.clone(), priority)
                        .map(|_| interface)
                        .map_err(|e| e.into())
                }
            }
        } else {
            Err(InterfaceCreateError::InterfaceTypeDoesNotExist)
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
    ) -> Result<Rc<ComInterface>, InterfaceCreateError> {
        info!("creating interface {interface_type}");
        if let Some(factory) = self.interface_factories.get(interface_type) {
            match factory {
                SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(
                    sync_factory,
                ) => {
                    let interface = ComInterface::create_from_sync_factory_fn(
                        *sync_factory,
                        setup_data,
                    )?;
                    self.add_interface(interface.clone(), priority)
                        .map(|_| interface)
                        .map_err(|e| e.into())
                }
                SyncOrAsyncComInterfaceImplementationFactoryFn::Async(_) => Err(
                    InterfaceCreateError::InterfaceCreationRequiresAsyncContext,
                ),
            }
        } else {
            Err(InterfaceCreateError::InterfaceTypeDoesNotExist)
        }
    }

    /// Checks if the interface with the given UUID exists in the manager
    pub fn has_interface(&self, interface_uuid: &ComInterfaceUUID) -> bool {
        self.interfaces.contains_key(interface_uuid)
    }

    /// Returns the com interface for a given UUID
    /// The interface is returned as a dynamic trait object
    pub fn try_dyn_interface_by_uuid(
        &self,
        uuid: &ComInterfaceUUID,
    ) -> Option<Rc<ComInterface>> {
        self.interfaces
            .get(uuid)
            .map(|(interface, _)| interface.clone())
    }

    /// Returns the com interface for a given UUID
    /// The interface must be registered in the ComHub,
    /// otherwise a panic will be triggered
    pub(crate) fn dyn_interface_by_uuid(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Rc<ComInterface> {
        self.try_dyn_interface_by_uuid(interface_uuid)
            .unwrap_or_else(|| {
                core::panic!("Interface for uuid {interface_uuid} not found")
            })
    }

    /// Adds an interface to the manager, checking for duplicates
    pub fn add_interface(
        &mut self,
        interface: Rc<ComInterface>,
        priority: InterfacePriority,
    ) -> Result<(), InterfaceAddError> {
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

        self.interfaces.insert(uuid, (interface.clone(), priority));
        Ok(())
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
    pub async fn remove_interface(
        &mut self,
        interface_uuid: ComInterfaceUUID,
    ) -> Result<(), ComHubError> {
        info!("Removing interface {interface_uuid}");
        let interface = self
            .interfaces
            .get_mut(&interface_uuid.clone())
            .ok_or(ComHubError::InterfaceDoesNotExist)?
            .0
            .clone();
        {
            // Async close the interface (stop tasks, server, cleanup internal data)
            interface.close();
            // TODO: await until closed asynchronously?
        }

        self.cleanup_interface(&interface_uuid)
            .ok_or(ComHubError::InterfaceDoesNotExist)?;

        Ok(())
    }

    /// The internal cleanup function that removes the interface from the hub
    /// and disabled the default interface if it was set to this interface
    fn cleanup_interface(
        &mut self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Option<Rc<ComInterface>> {
        Some(self.interfaces.remove(interface_uuid).or(None)?.0)
    }

    /// Handles interface events received from interfaces
    pub fn handle_interface_event(
        &mut self,
        interface_uuid: &ComInterfaceUUID,
        event: ComInterfaceEvent,
    ) {
        if let ComInterfaceEvent::Destroyed = event {
            // FIXME should probably do more cleanup here, but this was what com hub did before
            self.cleanup_interface(interface_uuid);
        }
    }
}
