use crate::{
    collections::HashMap,
    network::{
        com_hub::{
            InterfacePriority,
            errors::{ComInterfaceCreateError, InterfaceAddError},
        },
        com_interfaces::com_interface::{
            ComInterfaceUUID,
            factory::{
                ComInterfaceAsyncFactory, ComInterfaceConfiguration,
                ComInterfaceSyncFactory,
            },
            properties::{
                ComInterfaceProperties, InterfaceDirection, ReconnectionConfig,
            },
        },
    },
    prelude::*,
    values::value::Value,
};
use core::{
    cell::{Ref, RefCell},
    pin::Pin,
};
use futures::channel::oneshot;
use log::info;

type InterfaceMap = HashMap<ComInterfaceUUID, InterfaceInfo>;

pub type SyncComInterfaceImplementationFactoryFn =
    fn(
        setup_data: Value,
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
    fn(setup_data: Value) -> ComInterfaceAsyncFactoryResult;

pub type DynInterfaceImplementationFactoryFn =
    Rc<dyn Fn(Value) -> ComInterfaceAsyncFactoryResult>;

#[derive(Clone)]
pub enum SyncOrAsyncComInterfaceImplementationFactoryFn {
    Sync(SyncComInterfaceImplementationFactoryFn),
    Async(AsyncComInterfaceImplementationFactoryFn),
    Dyn(DynInterfaceImplementationFactoryFn),
}

#[derive(Debug)]
pub struct InterfaceCloseInfo {
    pub closed_sender: oneshot::Sender<()>,
}

pub type InterfaceCloseReceiver = oneshot::Receiver<InterfaceCloseInfo>;

#[derive(Debug)]
pub struct InterfaceInfo {
    pub properties: Rc<ComInterfaceProperties>,
    pub priority: InterfacePriority,
    pub is_waiting_for_socket_connections: bool,
    /// Original setup data that was used to create this interface, needed for reconnect
    pub original_setup_data: Option<Value>,
    pub close_sender: Option<oneshot::Sender<InterfaceCloseInfo>>,
}

#[derive(Default)]
pub struct ComInterfaceManager {
    /// a list of all available interface factories, keyed by their interface type
    pub interface_factories: RefCell<
        HashMap<String, SyncOrAsyncComInterfaceImplementationFactoryFn>,
    >,

    /// a mapping of interface type -> stored reconnect config
    pub reconnect_configs: RefCell<HashMap<String, ReconnectionConfig>>,

    /// a mapping of interface type -> fn that generates the default com interface properties
    pub interface_default_property_factories:
        RefCell<HashMap<String, fn() -> ComInterfaceProperties>>,

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
            interface_type.clone(),
            SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(T::factory),
        );
        self.reconnect_configs
            .borrow_mut()
            .insert(interface_type.clone(), T::get_reconnection_config());
        self.interface_default_property_factories
            .borrow_mut()
            .insert(interface_type, T::get_default_properties);
    }

    /// Registers a new async interface factory for a specific interface implementation.
    /// This allows the ComHub to create new instances of the interface on demand.
    pub fn register_async_interface_factory<T: ComInterfaceAsyncFactory>(
        &self,
    ) {
        let interface_type = T::get_default_properties().interface_type;
        self.interface_factories.borrow_mut().insert(
            interface_type.clone(),
            SyncOrAsyncComInterfaceImplementationFactoryFn::Async(T::factory),
        );
        self.reconnect_configs
            .borrow_mut()
            .insert(interface_type.clone(), T::get_reconnection_config());
        self.interface_default_property_factories
            .borrow_mut()
            .insert(interface_type, T::get_default_properties);
    }

    /// Registers a new custom async interface factory for a specific interface type.
    /// This allows the ComHub to create new instances of the interface on demand.
    pub fn register_dyn_interface_factory(
        &self,
        interface_type: String,
        factory: DynInterfaceImplementationFactoryFn,
        reconnection_config: ReconnectionConfig,
        get_default_properties: fn() -> ComInterfaceProperties,
    ) {
        self.interface_factories.borrow_mut().insert(
            interface_type.clone(),
            SyncOrAsyncComInterfaceImplementationFactoryFn::Dyn(factory),
        );
        self.reconnect_configs
            .borrow_mut()
            .insert(interface_type.clone(), reconnection_config);
        self.interface_default_property_factories
            .borrow_mut()
            .insert(interface_type, get_default_properties);
    }

    /// Creates a new interface instance using the registered factory
    /// for the specified interface type if it exists.
    /// The interface is opened and added to the ComHub.
    pub async fn create_and_add_interface(
        &self,
        interface_type: &str,
        setup_data: Value,
        priority: InterfacePriority,
    ) -> Result<
        (ComInterfaceConfiguration, InterfaceCloseReceiver),
        ComInterfaceCreateError,
    > {
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
                            &factory,
                            setup_data.clone(),
                        )
                        .await?;
                    self.add_interface(
                        com_interface_configuration.uuid(),
                        com_interface_configuration.properties.clone(),
                        priority,
                        Some(setup_data),
                    )
                    .map_err(|e| e.into())
                    .map(|close_receiver| {
                        (com_interface_configuration, close_receiver)
                    })
                }
            }
        } else {
            Err(ComInterfaceCreateError::InterfaceTypeNotRegistered(
                interface_type.to_string(),
            ))
        }
    }

    /// Get the registered [ReconnectionConfig] for the given interface type
    /// Defaults to [ReconnectionConfig::NoReconnect] if none is registered
    pub fn get_reconnection_config(
        &self,
        interface_type: &str,
    ) -> ReconnectionConfig {
        let reconnect_config =
            self.reconnect_configs.borrow().get(interface_type).cloned();
        reconnect_config.unwrap_or(ReconnectionConfig::NoReconnect)
    }

    /// Get the default [ComInterfaceProperties] for the given interface type if found
    pub fn get_interface_default_properties(
        &self,
        interface_type: &str,
    ) -> Option<ComInterfaceProperties> {
        self.interface_default_property_factories
            .borrow()
            .get(interface_type)
            .map(|factory| factory())
    }

    /// Creates a new interface instance using the registered sync factory
    /// for the specified interface type if it exists.
    /// If the factory is async, an error is returned.
    /// The interface is opened and added to the ComHub.
    pub fn create_and_add_interface_sync(
        &self,
        interface_type: &str,
        setup_data: Value,
        priority: InterfacePriority,
    ) -> Result<
        (ComInterfaceConfiguration, InterfaceCloseReceiver),
        ComInterfaceCreateError,
    > {
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
                            setup_data.clone(),
                        )?;
                    self.add_interface(com_interface_configuration.uuid(), com_interface_configuration.properties.clone(), priority, Some(setup_data))
                        .map_err(|e| e.into())
                        .map(|close_receiver| (com_interface_configuration, close_receiver))
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
        setup_data: Value,
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
        setup_data: Value,
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
    pub fn try_get_interface_properties_by_uuid(
        &self,
        uuid: &ComInterfaceUUID,
    ) -> Option<Rc<ComInterfaceProperties>> {
        let interfaces = self.interfaces.borrow();
        interfaces
            .get(uuid)
            .map(|InterfaceInfo { properties, .. }| properties.clone())
    }

    /// Returns the com interface info for a given UUID
    pub fn try_get_interface_info_by_uuid(
        &self,
        uuid: &ComInterfaceUUID,
    ) -> Option<Ref<'_, InterfaceInfo>> {
        Ref::filter_map(self.interfaces.borrow(), |interfaces| {
            interfaces.get(uuid)
        })
        .ok()
    }

    /// Returns the com interface  properties for a given UUID
    /// The interface must be registered in the ComHub,
    /// otherwise a panic will be triggered
    pub fn get_interface_properties_by_uuid(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Rc<ComInterfaceProperties> {
        self.try_get_interface_properties_by_uuid(interface_uuid)
            .unwrap_or_else(|| {
                core::panic!("Interface for uuid {interface_uuid} not found")
            })
    }

    /// Checks if the interface with the given UUID is currently waiting for a socket connection
    pub fn is_interface_waiting_for_socket_connections(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> bool {
        if let Some(interface_info) =
            self.interfaces.borrow().get(interface_uuid)
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
        if let Some(interface_info) =
            self.interfaces.borrow_mut().get_mut(interface_uuid)
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
        original_setup_data: Option<Value>,
    ) -> Result<InterfaceCloseReceiver, InterfaceAddError> {
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

        let (close_sender, close_receiver) = oneshot::channel();

        self.interfaces.borrow_mut().insert(
            uuid.clone(),
            InterfaceInfo {
                properties,
                priority,
                original_setup_data,
                is_waiting_for_socket_connections: true,
                close_sender: Some(close_sender),
            },
        );

        Ok(close_receiver)
    }

    /// Removes an interface by triggering the close signal
    pub async fn remove_interface(&self, uuid: &ComInterfaceUUID) {
        let closed_receiver = self.trigger_remove_interface(uuid);
        // wait for the interface to confirm it has been closed
        let _ = closed_receiver.await;
    }

    pub fn trigger_remove_interface(
        &self,
        uuid: &ComInterfaceUUID,
    ) -> oneshot::Receiver<()> {
        let (closed_sender, closed_receiver) = oneshot::channel();
        let close_sender = self
            .interfaces
            .borrow_mut()
            .get_mut(uuid)
            .unwrap()
            .close_sender
            .take()
            .unwrap();
        close_sender
            .send(InterfaceCloseInfo { closed_sender })
            .unwrap();
        closed_receiver
    }

    /// The internal cleanup function that removes the interface from the hub
    /// and disabled the default interface if it was set to this interface
    pub(crate) fn cleanup_interface(
        &self,
        interface_uuid: &ComInterfaceUUID,
    ) -> Result<InterfaceInfo, ()> {
        self.interfaces
            .borrow_mut()
            .remove(interface_uuid)
            .ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        network::com_interfaces::com_interface::factory::{
            SendCallback, SendSuccess, SocketConfiguration, SocketProperties,
        },
        prelude::*,
        runtime::cache::shared_references_cache::SharedReferencesCache,
        values::value::Value,
    };
    use datex_macros_internal::Datex;

    #[derive(Datex, Clone)]
    #[datex(structural)]
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
    fn create_interface_from_sync_factory() {
        let interface_manager = ComInterfaceManager::default();

        interface_manager.register_sync_interface_factory::<MockSetupData>();
        let setup_data = MockSetupData {
            name: "test-interface".to_string(),
        };
        let (com_interface_configuration, _) = interface_manager
            .create_and_add_interface_sync(
                "mock",
                Value::native(setup_data),
                InterfacePriority::None,
            )
            .unwrap();

        assert_eq!(
            com_interface_configuration.properties.channel,
            "mock".to_string()
        );

        assert_eq!(
            com_interface_configuration.properties.name,
            Some("test-interface".to_string())
        );

        // Clean up
        interface_manager
            .cleanup_interface(&com_interface_configuration.uuid())
            .unwrap();

        // Verify removal
        assert!(
            !interface_manager
                .has_interface(&com_interface_configuration.uuid())
        );
    }

    #[tokio::test]
    async fn create_interface_from_async_factory() {
        let interface_manager = ComInterfaceManager::default();

        interface_manager.register_async_interface_factory::<MockSetupData>();
        let setup_data = MockSetupData {
            name: "test-interface".to_string(),
        };
        let (com_interface_configuration, _) = interface_manager
            .create_and_add_interface(
                "mock",
                Value::native(setup_data),
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
            Some("test-interface".to_string())
        );

        // Clean up
        interface_manager
            .cleanup_interface(&com_interface_configuration.uuid())
            .unwrap();

        // Verify removal
        assert!(
            !interface_manager
                .has_interface(&com_interface_configuration.uuid())
        );
    }

    #[tokio::test]
    async fn create_interface_from_dyn_factory() {
        let interface_manager = ComInterfaceManager::default();
        let dyn_factory: DynInterfaceImplementationFactoryFn =
            Rc::new(|setup_data: Value| {
                Box::pin(async move {
                    let setup =
                        setup_data.try_into_value::<MockSetupData>().unwrap();
                    setup.create_interface()
                })
            });
        interface_manager.register_dyn_interface_factory(
            "mock".to_string(),
            dyn_factory,
            ReconnectionConfig::NoReconnect,
            ComInterfaceProperties::default,
        );
        let setup_data = MockSetupData {
            name: "test-interface".to_string(),
        };
        let (com_interface_configuration, _) = interface_manager
            .create_and_add_interface(
                "mock",
                Value::native(setup_data),
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
            Some("test-interface".to_string())
        );

        // Clean up
        interface_manager
            .cleanup_interface(&com_interface_configuration.uuid())
            .unwrap();

        // Verify removal
        assert!(
            !interface_manager
                .has_interface(&com_interface_configuration.uuid())
        );
    }
}
