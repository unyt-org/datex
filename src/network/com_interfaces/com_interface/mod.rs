use crate::network::com_interfaces::com_interface::{
    implementation::{
        ComInterfaceAsyncFactory, ComInterfaceImpl, ComInterfaceImplementation,
        ComInterfaceSyncFactory,
    },
    properties::InterfaceProperties,
    socket::{ComInterfaceSocketEvent, ComInterfaceSocketUUID},
    socket_manager::ComInterfaceSocketManager,
    state::{ComInterfaceState, ComInterfaceStateWrapper},
};

use crate::{
    network::com_hub::{
        errors::InterfaceCreateError,
        managers::interface_manager::{
            AsyncComInterfaceImplementationFactoryFn,
            SyncComInterfaceImplementationFactoryFn,
        },
    },
    stdlib::{
        cell::{Ref, RefCell, RefMut},
        rc::Rc,
        sync::{Arc, Mutex},
    },
    task::{UnboundedReceiver, UnboundedSender, create_unbounded_channel},
    utils::{once_consumer::OnceConsumer, uuid::UUID},
    values::value_container::ValueContainer,
};
use binrw::error::CustomError;
use core::fmt::{Debug, Display};
use tokio::sync::Notify;

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

#[derive(Debug, Clone)]
pub enum ComInterfaceImplEvent {
    SendBlock(Vec<u8>, ComInterfaceSocketUUID),
    Destroy,
    Reconnect,
}

#[derive(Debug)]
pub struct ComInterfaceInner {
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

    /// Receiver for interface implementation events (consumed by the implementation, sent by ComInterface)
    interface_impl_event_receiver:
        RefCell<OnceConsumer<UnboundedReceiver<ComInterfaceImplEvent>>>,

    /// Sender for interface implementation events (used by the ComInterface to send events to the implementation)
    interface_impl_event_sender:
        RefCell<UnboundedSender<ComInterfaceImplEvent>>,
}

impl ComInterfaceInner {
    pub fn init(
        state: ComInterfaceState,
        interface_properties: InterfaceProperties,
    ) -> Self {
        let (socket_event_sender, socket_event_receiver) =
            create_unbounded_channel::<ComInterfaceSocketEvent>();
        let (interface_event_sender, interface_event_receiver) =
            create_unbounded_channel::<ComInterfaceEvent>();

        let (interface_impl_event_sender, interface_impl_event_receiver) =
            create_unbounded_channel::<ComInterfaceImplEvent>();

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
            properties: Rc::new(RefCell::new(interface_properties)),

            interface_event_receiver: RefCell::new(OnceConsumer::new(
                interface_event_receiver,
            )),
            socket_event_receiver: RefCell::new(OnceConsumer::new(
                socket_event_receiver,
            )),
            interface_impl_event_receiver: RefCell::new(OnceConsumer::new(
                interface_impl_event_receiver,
            )),
            interface_impl_event_sender: RefCell::new(
                interface_impl_event_sender,
            ),
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

    pub fn take_interface_impl_event_receiver(
        &self,
    ) -> UnboundedReceiver<ComInterfaceImplEvent> {
        self.interface_impl_event_receiver.borrow_mut().consume()
    }

    pub fn state(&self) -> ComInterfaceState {
        self.state.try_lock().unwrap().get()
    }
    pub fn set_state(&self, new_state: ComInterfaceState) {
        self.state.try_lock().unwrap().set(new_state);
    }
    pub fn shutdown_signal(&self) -> Arc<Notify> {
        self.state.try_lock().unwrap().shutdown_signal().clone()
    }
}

/// A communication interface wrapper
/// which contains a concrete implementation of a com interface logic
pub struct ComInterface {
    pub(crate) inner: Rc<ComInterfaceInner>,
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
    pub fn shutdown_signal(&self) -> Arc<Notify> {
        self.inner.shutdown_signal()
    }

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
        com_interface.inner.properties.replace(properties);
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
        com_interface.inner.properties.replace(properties);
        Ok(com_interface)
    }

    fn create_headless() -> Rc<ComInterface> {
        Rc::new(ComInterface {
            inner: ComInterfaceInner::init(
                ComInterfaceState::Connected,
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
        com_interface.inner.properties.replace(properties);
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
        com_interface.inner.properties.replace(properties);
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
        self.inner.uuid.clone()
    }

    pub fn current_state(&self) -> ComInterfaceState {
        self.state().lock().unwrap().get()
    }

    pub fn state(&self) -> Arc<Mutex<ComInterfaceStateWrapper>> {
        self.inner.state.clone()
    }

    pub fn set_state(&self, new_state: ComInterfaceState) {
        self.inner.set_state(new_state);
    }

    pub fn properties(&self) -> Ref<'_, InterfaceProperties> {
        self.inner.properties.borrow()
    }

    /// Sends a block of data to the implementation to be transmitted over the specified socket
    /// Note: This method is non-blocking and returns immediately after queuing the send request
    /// If a block cannot be sent, the implementation should send it back to the com interface for retrying
    pub fn send_block(
        &self,
        block: &[u8],
        socket_uuid: ComInterfaceSocketUUID,
    ) {
        self.inner
            .interface_impl_event_sender
            .borrow_mut()
            .start_send(ComInterfaceImplEvent::SendBlock(
                block.to_vec(),
                socket_uuid,
            ))
            .unwrap();
    }

    pub fn reconnect(&self) {
        todo!()
    }

    /// Closes the communication interface and transitions it to the NotConnected state
    /// Note: This method is non-blocking and returns immediately after queuing the close request
    /// The actual closing of resources is handled asynchronously by the implementation
    pub fn close(&self) {
        self.inner
            .interface_impl_event_sender
            .borrow_mut()
            .start_send(ComInterfaceImplEvent::Destroy)
            .unwrap();
        self.set_state(ComInterfaceState::Destroyed);
    }

    pub fn info(&self) -> Rc<ComInterfaceInner> {
        self.inner.clone()
    }

    pub fn socket_manager(&self) -> Arc<Mutex<ComInterfaceSocketManager>> {
        self.info().socket_manager.clone()
    }

    pub fn take_interface_event_receiver(
        &self,
    ) -> UnboundedReceiver<ComInterfaceEvent> {
        self.inner.take_interface_event_receiver()
    }

    pub fn take_socket_event_receiver(
        &self,
    ) -> UnboundedReceiver<ComInterfaceSocketEvent> {
        self.inner.take_socket_event_receiver()
    }

    pub fn take_interface_impl_event_receiver(
        &self,
    ) -> UnboundedReceiver<ComInterfaceImplEvent> {
        self.inner.take_interface_impl_event_receiver()
    }
}
