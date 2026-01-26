use crate::network::{
    com_hub::managers::interfaces_manager::SyncOrAsyncComInterfaceImplementationFactoryFn,
    com_interfaces::com_interface::{
        factory::{ComInterfaceAsyncFactory, ComInterfaceSyncFactory},
        properties::InterfaceProperties,
        socket::{ComInterfaceSocketEvent, ComInterfaceSocketUUID},
        socket_manager::ComInterfaceSocketManager,
        state::{ComInterfaceState, ComInterfaceStateWrapper},
    },
};

use crate::{
    channel::mpsc::{
        UnboundedReceiver, UnboundedSender, create_unbounded_channel,
    },
    global::dxb_block::DXBBlock,
    network::{
        com_hub::{
            errors::ComInterfaceCreateError,
            managers::interfaces_manager::SyncComInterfaceImplementationFactoryFn,
        },
        com_interfaces::com_interface::properties::InterfaceDirection,
    },
    runtime::AsyncContext,
    std_sync::Mutex,
    stdlib::{
        cell::{Ref, RefCell},
        rc::Rc,
        sync::Arc,
        string::ToString,
    },
    utils::uuid::UUID,
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};
use core::fmt::{Debug, Display};
use crate::channel::futures_intrusive::ManualResetEvent;
use crate::stdlib::string::String;
use crate::stdlib::vec::Vec;
use crate::utils::async_callback::AsyncCallback;

pub mod error;
pub mod factory;
pub mod properties;
pub mod socket;
pub mod socket_manager;
pub mod state;

#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm_runtime", tsify(type = "string"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComInterfaceUUID(pub UUID);
impl Display for ComInterfaceUUID {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "com_interface::{}", self.0)
    }
}

impl TryFrom<String> for ComInterfaceUUID {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.strip_prefix("com_interface::").ok_or(())?;
        Ok(ComInterfaceUUID(UUID::from_string(value.to_string())))
    }
}

#[derive(Debug, Clone)]
pub enum ComInterfaceStateEvent {
    Connected,
    NotConnected,
    Destroyed,
}

#[derive(Debug, Clone)]
pub enum ComInterfaceEvent {
    SendBlock(DXBBlock, ComInterfaceSocketUUID),
    Destroy,
    Reconnect,
}

#[derive(Debug)]
pub struct ComInterfaceProxy {
    // Unique identifier for the interface
    pub uuid: ComInterfaceUUID,

    /// Connection state
    pub state: Arc<Mutex<ComInterfaceStateWrapper>>,

    /// Manager for sockets associated with this interface
    pub socket_manager: Arc<Mutex<ComInterfaceSocketManager>>,

    /// receiver for internal interface events that must be handled by the proxy (e.g. blocks to send)
    pub event_receiver: UnboundedReceiver<ComInterfaceEvent>,

    /// Async context that can be used to spawn async tasks
    pub async_context: AsyncContext,
}

type ComInterfaceProxyChannels = (
    UnboundedReceiver<ComInterfaceStateEvent>,
    UnboundedReceiver<ComInterfaceSocketEvent>,
    UnboundedSender<ComInterfaceEvent>,
);

type ComInterfaceProxyShared = (
    ComInterfaceUUID,
    Arc<Mutex<ComInterfaceStateWrapper>>,
    Arc<Mutex<ComInterfaceSocketManager>>,
);

impl ComInterfaceProxy {
    /// Creates a raw default ComInterfaceProxy instance along with its communication channels
    /// This can be used to connect a ComInterface implementation with the ComInterfaceProxy
    pub fn new_with_channels(
        async_context: AsyncContext,
    ) -> (Self, ComInterfaceProxyChannels) {
        // set up channels
        let (interface_state_event_sender, interface_state_event_receiver) =
            create_unbounded_channel::<ComInterfaceStateEvent>();

        let (socket_event_sender, socket_event_receiver) =
            create_unbounded_channel::<ComInterfaceSocketEvent>();

        let (interface_event_sender, interface_event_receiver) =
            create_unbounded_channel::<ComInterfaceEvent>();

        let uuid = ComInterfaceUUID(UUID::new());

        (
            Self {
                uuid: uuid.clone(),
                state: Arc::new(Mutex::new(ComInterfaceStateWrapper::new(
                    ComInterfaceState::Connected,
                    interface_state_event_sender,
                ))),
                socket_manager: Arc::new(Mutex::new(
                    ComInterfaceSocketManager::new_with_sender(
                        uuid,
                        socket_event_sender,
                        async_context.clone(),
                    ),
                )),
                event_receiver: interface_event_receiver,
                async_context,
            },
            (
                interface_state_event_receiver,
                socket_event_receiver,
                interface_event_sender,
            ),
        )
    }

    /// Creates a new ComInterface instance along with its proxy, configured with the specified properties
    pub fn create_interface(
        properties: InterfaceProperties,
        async_context: AsyncContext,
    ) -> (Self, ComInterfaceWithReceivers) {
        // Create a proxy for initialization
        let (com_interface_proxy, channels) =
            ComInterfaceProxy::new_with_channels(async_context);
        let com_interface_proxy_shared = com_interface_proxy.clone_shared();

        (
            com_interface_proxy,
            ComInterface::init_from_proxy_and_properties(
                com_interface_proxy_shared,
                channels,
                properties,
            ),
        )
    }

    fn clone_shared(&self) -> ComInterfaceProxyShared {
        (
            self.uuid.clone(),
            self.state.clone(),
            self.socket_manager.clone(),
        )
    }

    pub fn shutdown_receiver(&self) -> Arc<ManualResetEvent> {
        self.state.try_lock().unwrap().shutdown_receiver()
    }

    /// Creates and initializes a new socket and returns its UUID and sender
    /// Also registers an already known direct endpoint for the socket
    /// Locks the socket manager internally and calls the creation method
    pub fn create_and_init_socket_with_direct_endpoint(
        &self,
        direction: InterfaceDirection,
        channel_factor: u32,
        direct_endpoint: Endpoint,
    ) -> (ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>) {
        self.create_and_init_socket_with_optional_endpoint(
            direction,
            channel_factor,
            Some(direct_endpoint),
        )
    }

    /// Creates and initializes a new socket and returns its UUID and sender
    /// Locks the socket manager internally and calls the creation method
    pub fn create_and_init_socket(
        &self,
        direction: InterfaceDirection,
        channel_factor: u32,
    ) -> (ComInterfaceSocketUUID, AsyncCallback<Vec<u8>>) {
        self.create_and_init_socket_with_optional_endpoint(
            direction,
            channel_factor,
            None,
        )
    }

    pub fn create_and_init_socket_with_optional_endpoint(
        &self,
        direction: InterfaceDirection,
        channel_factor: u32,
        direct_endpoint: Option<Endpoint>,
    ) -> (ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>) {
        self.socket_manager
            .try_lock()
            .unwrap()
            .create_and_init_socket_with_optional_endpoint(
                direction,
                channel_factor,
                direct_endpoint,
            )
    }

    /// Couples two ComInterfaceProxy instances together, simulating a direct bidirectional read/write connection between them via
    /// a single socket.
    /// The socket manager and other internal components must be cloned before calling this method to still have access to them
    #[cfg(all(feature = "debug", feature = "std"))]
    pub fn couple_bidirectional(
        couple_a: (ComInterfaceProxy, Option<Endpoint>),
        couple_b: (ComInterfaceProxy, Option<Endpoint>),
    ) -> (ComInterfaceUUID, ComInterfaceUUID) {
        let (proxy_a, remote_endpoint_a) = couple_a;
        let (proxy_b, remote_endpoint_b) = couple_b;
        let uuid_a = proxy_a.uuid.clone();
        let uuid_b = proxy_b.uuid.clone();

        // Forward events from proxy A to proxy B
        let mut shutdown_signal_a = proxy_a.shutdown_receiver();
        let (_, mut socket_a_sender) = proxy_a
            .create_and_init_socket_with_optional_endpoint(
                InterfaceDirection::InOut,
                1,
                remote_endpoint_a,
            );

        // Forward events from proxy B to proxy A
        let mut shutdown_signal_b = proxy_b.shutdown_receiver();
        let (_, mut socket_b_sender) = proxy_b
            .create_and_init_socket_with_optional_endpoint(
                InterfaceDirection::InOut,
                1,
                remote_endpoint_b,
            );

        crate::task::spawn_with_panic_notify_default(async move {
            let mut event_receiver_a = proxy_a.event_receiver;
            loop {
                use async_select::select;

                select! {
                    Some(event) = event_receiver_a.next() => {
                        if let ComInterfaceEvent::SendBlock(block, _socket_uuid) = event {
                            // directly send the block to socket B
                            socket_b_sender.start_send(block.to_bytes()).unwrap();
                        }
                    }
                    _ = shutdown_signal_a.wait() => {
                        break;
                    }
                }
            }
        });
        crate::task::spawn_with_panic_notify_default(async move {
            let mut event_receiver_b = proxy_b.event_receiver;
            loop {
                use async_select::select;

                select! {
                    Some(event) = event_receiver_b.next() => {
                        if let ComInterfaceEvent::SendBlock(block, _socket_uuid) = event {
                            // directly send the block to socket A
                            socket_a_sender.start_send(block.to_bytes()).unwrap();
                        }
                    }
                    _ = shutdown_signal_b.wait() => {
                        break;
                    }
                }
            }
        });

        (uuid_a, uuid_b)
    }
}

/// A com interface that can be used by the com hub to manage communication
/// Implementations can be created using factory functions or setup data structs
pub struct ComInterface {
    // Unique identifier
    pub uuid: ComInterfaceUUID,

    /// Connection state
    pub state: Arc<Mutex<ComInterfaceStateWrapper>>,

    /// Manager for sockets associated with this interface
    pub socket_manager: Arc<Mutex<ComInterfaceSocketManager>>,

    /// Details about the interface
    /// FIXME make as Rc only, no RefCell needed
    pub properties: Rc<RefCell<InterfaceProperties>>,

    /// Sender for interface implementation events (used by the ComInterface to send events to the implementation)
    interface_event_sender: RefCell<UnboundedSender<ComInterfaceEvent>>,
}

pub type ComInterfaceReceivers = (
    UnboundedReceiver<ComInterfaceStateEvent>,
    UnboundedReceiver<ComInterfaceSocketEvent>,
);

pub type ComInterfaceWithReceivers = (ComInterface, ComInterfaceReceivers);

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
        factory_fn: &SyncComInterfaceImplementationFactoryFn,
        setup_data: ValueContainer,
        async_context: AsyncContext,
    ) -> Result<ComInterfaceWithReceivers, ComInterfaceCreateError> {
        // Create a proxy for initialization
        let (com_interface_proxy, channels) =
            ComInterfaceProxy::new_with_channels(async_context);
        let com_interface_proxy_shared = com_interface_proxy.clone_shared();

        // Create the implementation using the factory function
        let properties = factory_fn(setup_data, com_interface_proxy)?;

        Ok(ComInterface::init_from_proxy_and_properties(
            com_interface_proxy_shared,
            channels,
            properties,
        ))
    }

    pub async fn create_from_async_factory_fn(
        factory_fn: &SyncOrAsyncComInterfaceImplementationFactoryFn,
        setup_data: ValueContainer,
        async_context: AsyncContext,
    ) -> Result<ComInterfaceWithReceivers, ComInterfaceCreateError> {
        // Create a proxy for initialization
        let (com_interface_proxy, channels) =
            ComInterfaceProxy::new_with_channels(async_context);
        let com_interface_proxy_shared = com_interface_proxy.clone_shared();

        // Create the implementation using the factory function
        let properties = match factory_fn {
            SyncOrAsyncComInterfaceImplementationFactoryFn::Sync(sync_fn) => {
                sync_fn(setup_data, com_interface_proxy)?
            }
            SyncOrAsyncComInterfaceImplementationFactoryFn::Async(async_fn) => {
                async_fn(setup_data, com_interface_proxy).await?
            }
            SyncOrAsyncComInterfaceImplementationFactoryFn::Dyn(dyn_fn) => {
                dyn_fn(setup_data, com_interface_proxy).await?
            }
        };
        Ok(ComInterface::init_from_proxy_and_properties(
            com_interface_proxy_shared,
            channels,
            properties,
        ))
    }

    /// Creates a new ComInterface with the implementation of type T
    /// only works for sync factories
    pub fn create_sync_from_setup_data<T: ComInterfaceSyncFactory>(
        setup_data: T,
        async_context: AsyncContext,
    ) -> Result<ComInterfaceWithReceivers, ComInterfaceCreateError> {
        // Create a proxy for initialization
        let (com_interface_proxy, channels) =
            ComInterfaceProxy::new_with_channels(async_context);
        let com_interface_proxy_shared = com_interface_proxy.clone_shared();

        // Create the implementation using the factory function
        let properties = T::create_interface(setup_data, com_interface_proxy)?;

        Ok(ComInterface::init_from_proxy_and_properties(
            com_interface_proxy_shared,
            channels,
            properties,
        ))
    }

    /// Creates a new ComInterface with the implementation of type T
    /// only works for async factories
    pub async fn create_async_from_setup_data<T: ComInterfaceAsyncFactory>(
        setup_data: T,
        async_context: AsyncContext,
    ) -> Result<ComInterfaceWithReceivers, ComInterfaceCreateError> {
        // Create a proxy for initialization
        let (com_interface_proxy, channels) =
            ComInterfaceProxy::new_with_channels(async_context);
        let com_interface_proxy_shared = com_interface_proxy.clone_shared();

        // Create the implementation using the factory function
        let properties =
            T::create_interface(setup_data, com_interface_proxy).await?;

        Ok(ComInterface::init_from_proxy_and_properties(
            com_interface_proxy_shared,
            channels,
            properties,
        ))
    }

    pub fn uuid(&self) -> ComInterfaceUUID {
        self.uuid.clone()
    }

    pub fn current_state(&self) -> ComInterfaceState {
        self.state().try_lock().unwrap().get()
    }

    pub fn state(&self) -> Arc<Mutex<ComInterfaceStateWrapper>> {
        self.state.clone()
    }

    pub fn properties(&self) -> Ref<'_, InterfaceProperties> {
        self.properties.borrow()
    }

    /// Sends a block of data to the implementation to be transmitted over the specified socket
    /// Note: This method is non-blocking and returns immediately after queuing the send request
    /// If a block cannot be sent, the implementation should send it back to the com interface for retrying
    pub fn send_block(
        &self,
        block: DXBBlock,
        socket_uuid: ComInterfaceSocketUUID,
    ) {
        self.interface_event_sender
            .borrow_mut()
            .start_send(ComInterfaceEvent::SendBlock(block, socket_uuid))
            .unwrap();
    }

    pub fn reconnect(&self) {
        todo!()
    }

    /// Closes the communication interface and transitions it to the Destroyed state
    /// Note: This method is non-blocking and returns immediately after queuing the close request
    /// The actual closing of resources is handled asynchronously by the implementation
    pub fn destroy(&self) {
        self.interface_event_sender
            .borrow_mut()
            .start_send(ComInterfaceEvent::Destroy)
            .unwrap();
        self.state
            .try_lock()
            .unwrap()
            .set(ComInterfaceState::Destroyed);
    }

    pub fn socket_manager(&self) -> Arc<Mutex<ComInterfaceSocketManager>> {
        self.socket_manager.clone()
    }

    pub fn init_from_proxy_and_properties(
        proxy_shared: ComInterfaceProxyShared,
        channels: ComInterfaceProxyChannels,
        interface_properties: InterfaceProperties,
    ) -> ComInterfaceWithReceivers {
        (
            Self {
                uuid: proxy_shared.0,
                state: proxy_shared.1,
                socket_manager: proxy_shared.2,
                properties: Rc::new(RefCell::new(interface_properties)),
                interface_event_sender: RefCell::new(channels.2),
            },
            (channels.0, channels.1),
        )
    }

    pub fn set_state(&self, new_state: ComInterfaceState) {
        self.state.try_lock().unwrap().set(new_state);
    }
}
