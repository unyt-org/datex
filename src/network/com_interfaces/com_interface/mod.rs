use crate::network::com_interfaces::com_interface::implementation::{
    ComInterfaceAsyncFactory, ComInterfaceImpl, ComInterfaceImplementation,
    ComInterfaceSyncFactory,
};
use crate::network::com_interfaces::com_interface::properties::{
    InterfaceDirection, InterfaceProperties,
};
use crate::network::com_interfaces::com_interface::socket::{
    ComInterfaceSocket, ComInterfaceSocketEvent, ComInterfaceSocketUUID,
};
use crate::network::com_interfaces::com_interface::socket_manager::ComInterfaceSocketManager;
use crate::network::com_interfaces::com_interface::state::{
    ComInterfaceState, ComInterfaceStateWrapper,
};

use crate::network::com_hub::errors::InterfaceCreateError;
use crate::network::com_hub::managers::interface_manager::{
    AsyncComInterfaceImplementationFactoryFn,
    SyncComInterfaceImplementationFactoryFn,
};
use crate::stdlib::any::Any;
use crate::stdlib::cell::Ref;
use crate::stdlib::cell::RefCell;
use crate::stdlib::cell::RefMut;
use crate::stdlib::rc::Rc;
use crate::stdlib::sync::{Arc, Mutex};
use crate::task::{
    UnboundedReceiver, UnboundedSender, create_unbounded_channel,
};
use crate::utils::once_consumer::OnceConsumer;
use crate::utils::uuid::UUID;
use crate::values::core_values::endpoint::Endpoint;
use crate::values::value_container::ValueContainer;
use binrw::error::CustomError;
use core::cell::Cell;
use core::fmt::Debug;
use core::fmt::Display;
use core::pin::Pin;
use core::time::Duration;
use log::debug;

pub mod error;
pub mod implementation;
pub mod properties;
pub mod socket;
pub mod socket_manager;
pub mod state;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComInterfaceUUID(pub UUID);
impl Display for ComInterfaceUUID {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "ComInterface({})", self.0)
    }
}

impl ComInterfaceUUID {
    pub fn from_string(uuid: String) -> Self {
        ComInterfaceUUID(UUID::from_string(uuid))
    }
}

#[derive(Debug, Clone)]
pub enum ComInterfaceEvent {
    Connected,
    NotConnected,
    Destroyed,
}

#[derive(Debug)]
pub struct ComInterfaceInfo {
    // Unique identifier
    pub uuid: ComInterfaceUUID,

    /// Connection state
    pub state: Arc<Mutex<ComInterfaceStateWrapper>>,

    /// Manager for sockets associated with this interface
    pub socket_manager: Arc<Mutex<ComInterfaceSocketManager>>,

    /// Details about the interface
    pub properties: Rc<RefCell<InterfaceProperties>>,

    /// Receiver for interface events (consumed by ComHub)
    socket_event_receiver:
        RefCell<OnceConsumer<UnboundedReceiver<ComInterfaceSocketEvent>>>,

    /// Receiver for interface events (consumed by ComHub)
    interface_event_receiver:
        RefCell<OnceConsumer<UnboundedReceiver<ComInterfaceEvent>>>,
}

impl ComInterfaceInfo {
    pub fn init(
        state: ComInterfaceState,
        interface_properties: InterfaceProperties,
    ) -> Self {
        let (socket_event_sender, socket_event_receiver) =
            create_unbounded_channel::<ComInterfaceSocketEvent>();
        let (interface_event_sender, interface_event_receiver) =
            create_unbounded_channel::<ComInterfaceEvent>();
        let uuid = ComInterfaceUUID(UUID::new());
        Self {
            state: Arc::new(Mutex::new(ComInterfaceStateWrapper::new(
                state,
                interface_event_sender,
            ))),
            socket_manager: Arc::new(Mutex::new(
                ComInterfaceSocketManager::new_with_sender(
                    uuid.clone(),
                    socket_event_sender,
                ),
            )),
            uuid,
            interface_event_receiver: RefCell::new(OnceConsumer::new(
                interface_event_receiver,
            )),
            properties: Rc::new(RefCell::new(interface_properties)),
            socket_event_receiver: RefCell::new(OnceConsumer::new(
                socket_event_receiver,
            )),
        }
    }

    pub fn take_socket_event_receiver(
        &self,
    ) -> UnboundedReceiver<ComInterfaceSocketEvent> {
        self.socket_event_receiver.borrow_mut().consume()
    }
    pub fn take_interface_event_receiver(
        &self,
    ) -> UnboundedReceiver<ComInterfaceEvent> {
        self.interface_event_receiver.borrow_mut().consume()
    }

    pub fn state(&self) -> ComInterfaceState {
        self.state.try_lock().unwrap().get()
    }
    pub fn set_state(&self, new_state: ComInterfaceState) {
        self.state.try_lock().unwrap().set(new_state);
    }
}

/// A communication interface wrapper
/// which contains a concrete implementation of a com interface logic
pub struct ComInterface {
    pub(crate) info: Rc<ComInterfaceInfo>,
    pub(crate) implementation: RefCell<Option<Box<dyn ComInterfaceImpl>>>,
}

impl Debug for ComInterface {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ComInterface")
            .field("uuid", &self.uuid())
            .field("state", &self.current_state())
            .field("properties", &self.properties())
            .finish()
    }
}

impl ComInterface {
    /// Initializes a new ComInterface with a specified implementation as returned by the factory function
    pub fn create_from_sync_factory_fn(
        factory_fn: SyncComInterfaceImplementationFactoryFn,
        setup_data: ValueContainer,
    ) -> Result<Rc<ComInterface>, InterfaceCreateError> {
        // Create a headless ComInterface first
        let com_interface = Self::create_headless();

        // Create the implementation using the factory function
        let (implementation, properties) =
            factory_fn(setup_data, com_interface.clone())?;
        com_interface.set_implementation(implementation);
        com_interface.info.properties.replace(properties);
        Ok(com_interface)
    }

    pub async fn create_from_async_factory_fn(
        factory_fn: AsyncComInterfaceImplementationFactoryFn,
        setup_data: ValueContainer,
    ) -> Result<Rc<ComInterface>, InterfaceCreateError> {
        // Create a headless ComInterface first
        let com_interface = Self::create_headless();

        // Create the implementation using the factory function
        let (implementation, properties) =
            factory_fn(setup_data, com_interface.clone()).await?;
        com_interface.set_implementation(implementation);
        com_interface.info.properties.replace(properties);
        Ok(com_interface)
    }

    fn create_headless() -> Rc<ComInterface> {
        Rc::new(ComInterface {
            info: ComInterfaceInfo::init(
                ComInterfaceState::NotConnected,
                InterfaceProperties::default(),
            )
            .into(),
            implementation: RefCell::new(None),
        })
    }

    /// Creates a new ComInterface with the implementation of type T
    /// only works for sync factories
    pub fn create_sync_with_implementation<T>(
        setup_data: T::SetupData,
    ) -> Result<Rc<ComInterface>, InterfaceCreateError>
    where
        T: ComInterfaceImplementation + ComInterfaceSyncFactory,
    {
        // Create a headless ComInterface first
        let com_interface = Self::create_headless();

        // Create the implementation using the factory function
        let (implementation, properties) =
            T::create(setup_data, com_interface.clone())?;
        com_interface.set_implementation(Box::new(implementation));
        com_interface.info.properties.replace(properties);
        Ok(com_interface)
    }

    /// Creates a new ComInterface with the implementation of type T
    /// only works for async factories
    pub async fn create_async_with_implementation<T>(
        setup_data: T::SetupData,
    ) -> Result<Rc<ComInterface>, InterfaceCreateError>
    where
        T: ComInterfaceImplementation + ComInterfaceAsyncFactory,
    {
        // Create a headless ComInterface first
        let com_interface = Self::create_headless();

        // Create the implementation using the factory function
        let (implementation, properties) =
            T::create(setup_data, com_interface.clone()).await?;
        com_interface.set_implementation(Box::new(implementation));
        com_interface.info.properties.replace(properties);
        Ok(com_interface)
    }

    pub fn implementation_mut<T: ComInterfaceImpl>(&self) -> RefMut<'_, T> {
        RefMut::map(self.implementation.borrow_mut(), |opt| {
            opt.as_mut()
                .expect("ComInterface is not initialized")
                .as_any_mut()
                .downcast_mut::<T>()
                .expect("ComInterface implementation type mismatch")
        })
    }

    pub fn implementation<T: ComInterfaceImpl>(&self) -> Ref<'_, T> {
        Ref::map(self.implementation.borrow(), |opt| {
            opt.as_ref()
                .expect("ComInterface is not initialized")
                .as_any()
                .downcast_ref::<T>()
                .expect("ComInterface implementation type mismatch")
        })
    }

    /// Initializes a headless ComInterface with the provided implementation
    /// and upgrades it to an Initialized state.
    /// This can only be done once on a headless interface and will panic if attempted on an already initialized interface.
    pub(crate) fn set_implementation(
        &self,
        implementation: Box<dyn ComInterfaceImpl>,
    ) {
        match self.implementation.replace(Some(implementation)) {
            None => {
                // Successfully initialized
            }
            Some(_) => {
                panic!("ComInterface is already initialized");
            }
        }
    }

    pub fn uuid(&self) -> ComInterfaceUUID {
        self.info.uuid.clone()
    }

    pub fn current_state(&self) -> ComInterfaceState {
        self.state().lock().unwrap().get()
    }

    pub fn state(&self) -> Arc<Mutex<ComInterfaceStateWrapper>> {
        self.info.state.clone()
    }

    pub fn set_state(&self, new_state: ComInterfaceState) {
        self.info.set_state(new_state);
    }

    pub fn properties(&self) -> Ref<'_, InterfaceProperties> {
        self.info.properties.borrow()
    }

    pub async fn send_block(
        &self,
        block: &[u8],
        socket_uuid: ComInterfaceSocketUUID,
    ) -> bool {
        match self.implementation.borrow_mut().as_mut() {
            None => {
                panic!("Cannot send block on headless ComInterface");
            }
            Some(implementation) => {
                implementation.send_block(block, socket_uuid).await
            }
        }
    }

    pub async fn reconnect(&self) -> bool {
        todo!()
    }

    pub async fn close(&self) -> bool {
        self.set_state(ComInterfaceState::Closing);
        let result = match self.implementation.borrow_mut().as_mut() {
            None => {
                panic!("Cannot close headless ComInterface");
            }
            Some(implementation) => implementation.handle_destroy().await,
        };
        self.set_state(ComInterfaceState::NotConnected);
        result
    }

    pub fn info(&self) -> Rc<ComInterfaceInfo> {
        self.info.clone()
    }

    pub fn socket_manager(&self) -> Arc<Mutex<ComInterfaceSocketManager>> {
        self.info().socket_manager.clone()
    }

    pub fn take_interface_event_receiver(
        &self,
    ) -> UnboundedReceiver<ComInterfaceEvent> {
        self.info.take_interface_event_receiver()
    }

    pub fn take_socket_event_receiver(
        &self,
    ) -> UnboundedReceiver<ComInterfaceSocketEvent> {
        self.info.take_socket_event_receiver()
    }
}
