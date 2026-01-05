use crate::{
    collections::HashMap,
    network::{
        com_hub::{
            ComHubError, InterfacePriority,
            errors::{ComInterfaceCreateError, InterfaceAddError},
        },
        com_interfaces::com_interface::{
            ComInterfaceUUID,
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceConfiguration,
                ComInterfaceSyncFactory,
            },
            properties::{ComInterfaceProperties, InterfaceDirection},
        },
    },
    prelude::*,
    values::value_container::ValueContainer,
};
use core::{cell::RefCell, pin::Pin};
use log::info;

type InterfaceMap =
    HashMap<ComInterfaceUUID, InterfaceInfo>;

pub type SyncComInterfaceImplementationFactoryFn =
    fn(
        setup_data: ValueContainer,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError>;

pub type ComInterfaceAsyncFactoryResult = Pin<
    Box<
        dyn Future<
                Output = Result<
                    ComInterfaceConfiguration,
                    ComInterfaceCreateError,
                >,
            > + 'static,
    >,
>;

pub type AsyncComInterfaceImplementationFactoryFn =
    fn(setup_data: ValueContainer) -> ComInterfaceAsyncFactoryResult;

pub type DynInterfaceImplementationFactoryFn =
    Rc<dyn Fn(ValueContainer) -> ComInterfaceAsyncFactoryResult>;

#[derive(Clone)]
pub enum SyncOrAsyncComInterfaceImplementationFactoryFn {
    Sync(SyncComInterfaceImplementationFactoryFn),
    Async(AsyncComInterfaceImplementationFactoryFn),
    Dyn(DynInterfaceImplementationFactoryFn),
}

#[derive(Debug)]
pub struct InterfaceInfo {
    pub properties: Rc<ComInterfaceProperties>,
    pub priority: InterfacePriority,
    pub is_waiting_for_socket_connections: bool,
}

#[derive(Default)]
pub struct ComInterfaceManager {
    /// a list of all available interface factories, keyed by their interface type
    pub interface_factories: RefCell<
        HashMap<String, SyncOrAsyncComInterfaceImplementationFactoryFn>,
    >,

    /// a list of all available interfaces, keyed by their UUID
    pub interfaces: RefCell<InterfaceMap>,
}

/// Manages the registered interfaces and their factories
/// Allows creating, adding, removing and querying interfaces
/// Also handles interface events (lifecycle management)
impl ComInterfaceManager {
    /// Registers a new sync interface factory for a specific interface implementation.
    /// This allows the ComHub to create new instances of the interface on demand.
    pub fn register_sync_interface_factory<T: ComInterfaceSyncFactory>(&self) {
        let interface_type = T::get_default_properties().interface_type;
        self.interface_factories.borrow_mut().insert(
            interface_type,
            SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(T::factory),
        );
    }

    /// Registers a new async interface factory for a specific interface implementation.
    /// This allows the ComHub to create new instances of the interface on demand.
    pub fn register_async_interface_factory<T: ComInterfaceAsyncFactory>(
        &self,
    ) {
        let interface_type = T::get_default_properties().interface_type;
        self.interface_factories.borrow_mut().insert(
            interface_type,
            SyncOrAsyncComInterfaceImplementationFactoryFn::Async(T::factory),
        );
    }

    /// Registers a new custom async interface factory for a specific interface type.
    /// This allows the ComHub to create new instances of the interface on demand.
    pub fn register_dyn_interface_factory(
        &self,
        interface_type: String,
        factory: DynInterfaceImplementationFactoryFn,
    ) {
        self.interface_factories.borrow_mut().insert(
            interface_type,
            SyncOrAsyncComInterfaceImplementationFactoryFn::Dyn(factory),
        );
    }

    /// Creates a new interface instance using the registered factory
    /// for the specified interface type if it exists.
    /// The interface is opened and added to the ComHub.
    pub async fn create_and_add_interface(
        &self,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        info!("creating interface {interface_type}");
        let factory = self
            .interface_factories
            .borrow()
            .get(interface_type)
            .cloned();
        if let Some(factory) = factory {
            match factory {
                SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(_) => self
                    .create_and_add_interface_sync(
                        interface_type,
                        setup_data,
                        priority,
                    ),
                SyncOrAsyncComInterfaceImplementationFactoryFn::Async(_)
                | SyncOrAsyncComInterfaceImplementationFactoryFn::Dyn(_) => {
                    let com_interface_configuration =
                        Self::create_interface_from_async_factory_fn(
                            &factory, setup_data,
                        )
                        .await?;
                    self.add_interface(
                        com_interface_configuration.uuid(),
                        com_interface_configuration.properties.clone(),
                        priority,
                    )
                    .map_err(|e| e.into())
                    .map(|_| com_interface_configuration)
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
        &self,
        interface_type: &str,
        setup_data: ValueContainer,
        priority: InterfacePriority,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        info!("creating interface sync {interface_type}");
        if let Some(factory) =
            self.interface_factories.borrow().get(interface_type)
        {
            match factory {
                SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(
                    sync_factory,
                ) => {
                    let com_interface_configuration =
                        Self::create_interface_from_sync_factory_fn(
                            sync_factory,
                            setup_data,
                        )?;
                    self.add_interface(com_interface_configuration.uuid(), com_interface_configuration.properties.clone(), priority)
                        .map_err(|e| e.into())
                        .map(|_| com_interface_configuration)
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

    async fn create_interface_from_async_factory_fn(
        factory_fn: &SyncOrAsyncComInterfaceImplementationFactoryFn,
        setup_data: ValueContainer,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        // Create the implementation using the factory function
        match factory_fn {
            SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(sync_fn) => {
                sync_fn(setup_data)
            }
            SyncOrAsyncComInterfaceImplementationFactoryFn::Async(async_fn) => {
                async_fn(setup_data).await
            }
            SyncOrAsyncComInterfaceImplementationFactoryFn::Dyn(dyn_fn) => {
                dyn_fn(setup_data).await
            }
        }
    }

    /// Initializes a new ComInterface with a specified implementation as returned by the factory function
    fn create_interface_from_sync_factory_fn(
        factory_fn: &SyncComInterfaceImplementationFactoryFn,
        setup_data: ValueContainer,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        // Create the implementation using the factory function
        factory_fn(setup_data)
    }

    /// Creates a new ComInterface with the implementation of type T
    /// only works for sync factories
    pub fn create_interface_sync_from_setup_data<T: ComInterfaceSyncFactory>(
        setup_data: T,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        // Create the implementation using the factory function
        T::create_interface(setup_data)
    }

    /// Creates a new ComInterface with the implementation of type T
    /// only works for async factories
    pub async fn create_interface_async_from_setup_data<
        T: ComInterfaceAsyncFactory,
    >(
        setup_data: T,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        // Create the implementation using the factory function
        T::create_interface(setup_data).await
    }

    /// Checks if the interface with the given UUID exists in the manager
    pub fn has_interface(&self, interface_uuid: &ComInterfaceUUID) -> bool {
        self.interfaces.borrow().contains_key(interface_uuid)
    }

    /// Returns the com interface properties for a given UUID
    /// The interface is returned as a dynamic trait object
    pub fn try_get_interface_by_uuid(
        &self,
        uuid: &ComInterfaceUUID,
    ) -> Option<Rc<ComInterfaceProperties>> {
        let interfaces = self.interfaces.borrow();
        interfaces
            .get(uuid)
            .map(|InterfaceInfo{properties, ..}| properties.clone())
    }

    /// Returns the com interface  properties for a given UUID
    /// The interface must be registered in the ComHub,
    /// otherwise a panic will be triggered
    pub fn get_interface_by_uuid(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Rc<ComInterfaceProperties> {
        self.try_get_interface_by_uuid(interface_uuid)
            .unwrap_or_else(|| {
                core::panic!("Interface for uuid {interface_uuid} not found")
            })
    }

    /// Checks if the interface with the given UUID is currently waiting for a socket connection
    pub fn is_interface_waiting_for_socket_connections(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> bool {
        if let Some(interface_info) = self
            .interfaces
            .borrow()
            .get(interface_uuid)
        {
            interface_info.is_waiting_for_socket_connections
        } else {
            core::panic!("Interface for uuid {interface_uuid} not found");
        }
    }

    /// Sets the waiting for socket connection state of the interface with the given UUID
    pub fn set_interface_waiting_for_socket_connections(
        &self,
        interface_uuid: &ComInterfaceUUID,
        is_waiting: bool,
    ) {
        if let Some(interface_info) = self
            .interfaces
            .borrow_mut()
            .get_mut(interface_uuid)
        {
            interface_info.is_waiting_for_socket_connections = is_waiting;
        } else {
            core::panic!("Interface for uuid {interface_uuid} not found");
        }
    }

    /// Adds an interface to the manager, checking for duplicates
    pub fn add_interface(
        &self,
        uuid: ComInterfaceUUID,
        properties: Rc<ComInterfaceProperties>,
        priority: InterfacePriority,
    ) -> Result<(), InterfaceAddError> {
        if self.interfaces.borrow().contains_key(&uuid) {
            return Err(InterfaceAddError::InterfaceAlreadyExists);
        }

        // make sure the interface can send if a priority is set
        if priority != InterfacePriority::None
            && properties.direction == InterfaceDirection::In
        {
            return Err(
                InterfaceAddError::InvalidInterfaceDirectionForFallbackInterface,
            );
        }

        self.interfaces
            .borrow_mut()
            .insert(uuid.clone(), InterfaceInfo{
                properties,
                priority,
                is_waiting_for_socket_connections: true
            });

        Ok(())
    }

    /// Returns the priority of the interface with the given UUID
    pub fn interface_priority(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Option<InterfacePriority> {
        self.interfaces
            .borrow()
            .get(interface_uuid)
            .map(|(InterfaceInfo { priority, ..})| *priority)
    }

    /// User can proactively remove an interface from the hub.
    /// This will destroy the interface and it's sockets (perform deep cleanup)
    pub fn destroy_interface(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Result<(), ComHubError> {
        info!("Removing interface {interface_uuid}");
        // let _interface = &mut self
        //     .interfaces
        //     .borrow_mut()
        //     .get_mut(interface_uuid)
        //     .ok_or(ComHubError::InterfaceDoesNotExist)?
        //     .0;
        // TODO: destroy?

        self.cleanup_interface(interface_uuid)?;

        Ok(())
    }

    /// The internal cleanup function that removes the interface from the hub
    /// and disabled the default interface if it was set to this interface
    fn cleanup_interface(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Result<(), ComHubError> {
        self.interfaces
            .borrow_mut()
            .remove(interface_uuid)
            .ok_or(ComHubError::InterfaceDoesNotExist)
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::com_interfaces::com_interface::factory::{
        SendCallback, SendSuccess, SocketConfiguration, SocketProperties,
    };

    use crate::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    struct MockSetupData {
        name: String,
    }

    impl MockSetupData {
        fn create_interface(
            self,
        ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError>
        {
            Ok(ComInterfaceConfiguration::new_single_socket(
                ComInterfaceProperties {
                    name: Some(self.name),
                    ..<MockSetupData as ComInterfaceSyncFactory>::get_default_properties()
                },
                SocketConfiguration::new_in_out(
                    SocketProperties::new(InterfaceDirection::InOut, 1),
                    async gen move {
                        // mock socket incoming data iterator
                        loop {
                            yield Ok(vec![]);
                        }
                    },
                    SendCallback::new_sync(|_block| Ok(SendSuccess::Sent))
                )
            ))
        }
    }

    impl ComInterfaceSyncFactory for MockSetupData {
        fn create_interface(
            self,
        ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError>
        {
            self.create_interface()
        }

        fn get_default_properties() -> ComInterfaceProperties {
            ComInterfaceProperties {
                interface_type: "mock".to_string(),
                channel: "mock".to_string(),
                ..Default::default()
            }
        }
    }

    impl ComInterfaceAsyncFactory for MockSetupData {
        fn create_interface(self) -> ComInterfaceAsyncFactoryResult {
            Box::pin(async move { self.create_interface() })
        }

        fn get_default_properties() -> ComInterfaceProperties {
            ComInterfaceProperties {
                interface_type: "mock".to_string(),
                channel: "mock".to_string(),
                ..Default::default()
            }
        }
    }

    #[test]
    fn test_create_interface_from_sync_factory() {
        let interface_manager = ComInterfaceManager::default();

        interface_manager.register_sync_interface_factory::<MockSetupData>();
        let setup_data = MockSetupData {
            name: "test_interface".to_string(),
        };
        let com_interface_configuration = interface_manager
            .create_and_add_interface_sync(
                "mock",
                ValueContainer::from_serializable(&setup_data).unwrap(),
                InterfacePriority::None,
            )
            .unwrap();

        assert_eq!(
            com_interface_configuration.properties.channel,
            "mock".to_string()
        );

        assert_eq!(
            com_interface_configuration.properties.name,
            Some("test_interface".to_string())
        );

        // Clean up
        interface_manager
            .destroy_interface(&com_interface_configuration.uuid())
            .unwrap();

        // Verify removal
        assert!(
            !interface_manager
                .has_interface(&com_interface_configuration.uuid())
        );
    }

    #[tokio::test]
    async fn test_create_interface_from_async_factory() {
        let interface_manager = ComInterfaceManager::default();

        interface_manager.register_async_interface_factory::<MockSetupData>();
        let setup_data = MockSetupData {
            name: "test_interface".to_string(),
        };
        let com_interface_configuration = interface_manager
            .create_and_add_interface(
                "mock",
                ValueContainer::from_serializable(&setup_data).unwrap(),
                InterfacePriority::None,
            )
            .await
            .unwrap();

        assert_eq!(
            com_interface_configuration.properties.channel,
            "mock".to_string()
        );

        assert_eq!(
            com_interface_configuration.properties.name,
            Some("test_interface".to_string())
        );

        // Clean up
        interface_manager
            .destroy_interface(&com_interface_configuration.uuid())
            .unwrap();

        // Verify removal
        assert!(
            !interface_manager
                .has_interface(&com_interface_configuration.uuid())
        );
    }

    #[tokio::test]
    async fn test_create_interface_from_dyn_factory() {
        let interface_manager = ComInterfaceManager::default();
        let dyn_factory: DynInterfaceImplementationFactoryFn =
            Rc::new(|setup_data: ValueContainer| {
                Box::pin(async move {
                    let setup: MockSetupData =
                        setup_data.cast_to_deserializable().unwrap();
                    setup.create_interface()
                })
            });
        interface_manager
            .register_dyn_interface_factory("mock".to_string(), dyn_factory);
        let setup_data = MockSetupData {
            name: "test_interface".to_string(),
        };
        let com_interface_configuration = interface_manager
            .create_and_add_interface(
                "mock",
                ValueContainer::from_serializable(&setup_data).unwrap(),
                InterfacePriority::None,
            )
            .await
            .unwrap();

        assert_eq!(
            com_interface_configuration.properties.channel,
            "mock".to_string()
        );

        assert_eq!(
            com_interface_configuration.properties.name,
            Some("test_interface".to_string())
        );

        // Clean up
        interface_manager
            .destroy_interface(&com_interface_configuration.uuid())
            .unwrap();

        // Verify removal
        assert!(
            !interface_manager
                .has_interface(&com_interface_configuration.uuid())
        );
    }
}
