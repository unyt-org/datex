use core::async_iter::AsyncIterator;
use core::future::poll_fn;
use core::pin::Pin;
use core::fmt::Debug;
use std::sync::Arc;
use crate::stdlib::rc::Rc;
pub(crate) use crate::network::com_hub::managers::interfaces_manager::ComInterfaceAsyncFactoryResult;
use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            properties::ComInterfaceProperties,
        },
    },
    serde::deserializer::from_value_container,
    values::value_container::ValueContainer,
};
use serde::de::DeserializeOwned;
use crate::global::dxb_block::DXBBlock;
use crate::network::com_hub::InterfacePriority;
use crate::network::com_interfaces::com_interface::ComInterfaceUUID;
use crate::network::com_interfaces::com_interface::properties::InterfaceDirection;
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;
use crate::stdlib::boxed::Box;
use crate::utils::async_callback::AsyncCallback;
use crate::utils::time::Time;
use crate::utils::uuid::UUID;
use crate::values::core_values::endpoint::Endpoint;


// utility function for async next
pub async fn async_next_pin_box<I>(iter: &mut Pin<Box<I>>) -> Option<I::Item>
where
    I: AsyncIterator + ?Sized,
{
    poll_fn(|cx| {
        iter.as_mut().poll_next(cx)
    })
        .await
}


pub type NewSocketsIterator = Pin<Box<dyn AsyncIterator<Item = Result<SocketConfiguration, ()>> + Send>>;

#[derive(Debug, Clone)]
pub struct SocketProperties {
    pub direction: InterfaceDirection,
    pub channel_factor: u32,
    pub direct_endpoint: Option<Endpoint>,
    pub connection_timestamp: u64,
    uuid: ComInterfaceSocketUUID,
}

impl SocketProperties {
    pub fn new(
        direction: InterfaceDirection,
        channel_factor: u32,
    ) -> Self {
        SocketProperties {
            direction,
            channel_factor,
            direct_endpoint: None,
            connection_timestamp: Time::now(),
            uuid: ComInterfaceSocketUUID(UUID::new()),
        }
    }
    pub fn new_with_direct_endpoint(
        direction: InterfaceDirection,
        channel_factor: u32,
        endpoint: Endpoint,
    ) -> Self {
        SocketProperties {
            direction,
            channel_factor,
            direct_endpoint: Some(endpoint),
            connection_timestamp: Time::now(),
            uuid: ComInterfaceSocketUUID(UUID::new()),
        }
    }

    pub fn uuid(&self) -> ComInterfaceSocketUUID {
        self.uuid.clone()
    }
}

pub type SocketDataIterator = Pin<Box<dyn AsyncIterator<Item = Result<Vec<u8>, ()>> + Send>>;

pub struct SocketConfiguration {
    pub properties: SocketProperties,
    /// An asynchronous iterator that yields incoming data from the socket as Vec<u8>
    /// It is driven by the com hub to receive data from the socket
    pub iterator: Option<SocketDataIterator>,
    /// A callback that is called by the com hub to send data through the socket
    /// This can be either a synchronous or asynchronous callback depending on the interface implementation
    pub send_callback: Option<SendCallback>
}
impl SocketConfiguration {
    /// Creates a SocketDataIterator for a given socket with the provided parameters.
    /// Expects both an iterator for incoming data and a send callback for outgoing data.
    pub fn new<I>(
        socket_configuration: SocketProperties,
        iter: I,
        send_callback: SendCallback,
    ) -> Self
    where
        I: AsyncIterator<Item=Result<Vec<u8>, ()>> + Send + 'static,
    {
        SocketConfiguration {
            properties: socket_configuration,
            iterator: Some(Box::pin(iter)),
            send_callback: Some(send_callback),
        }
    }

    /// Creates a SocketDataIterator for a given socket with the provided parameters.
    /// Only handles incoming data; no send callback is provided.
    pub fn new_in<I>(
        socket_configuration: SocketProperties,
        maybe_iter: Option<I>,
    ) -> Self
    where
        I: AsyncIterator<Item=Result<Vec<u8>, ()>> + Send + 'static,
    {
        SocketConfiguration {
            properties: socket_configuration,
            iterator: maybe_iter.map(|it| Box::pin(it) as Pin<Box<dyn AsyncIterator<Item=Result<Vec<u8>, ()>> + Send>>),
            send_callback: None,
        }
    }

    /// Creates a SocketDataIterator for a given socket with the provided parameters.
    /// Only handles outgoing data; no incoming data iterator is provided.
    pub fn new_out(
        socket_configuration: SocketProperties,
        send_callback: SendCallback,
    ) -> Self {
        SocketConfiguration {
            properties: socket_configuration,
            iterator: None,
            send_callback: Some(send_callback),
        }
    }
}

/// A callback that is called by the com hub to send data through the interface.
#[derive(Clone)]
pub enum SendCallback {
    /// A synchronous send callback.
    /// The callback receives a DXBBlock and the UUID of the socket to send the data through.
    /// It returns a SendSuccess result which can contain already received data from the remote side.
    /// The failure case returns a SendFailure containing the original DXBBlock.
    Sync(Arc<dyn Fn(DXBBlock) -> Result<SendSuccess, SendFailure> + 'static + Send + Sync>),
    /// An asynchronous send callback.
    /// The callback receives a DXBBlock and the UUID of the socket to send the data through.
    /// It returns a future that resolves to a Result indicating success or failure.
    /// The success case does not return any data, as any received data should be handled
    /// through the receive iterator.
    /// The failure case returns a SendFailure containing the original DXBBlock.
    Async(AsyncCallback<DXBBlock, Result<(), SendFailure>>),
}

impl Debug for SendCallback {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SendCallback::Sync(_) => write!(f, "SendCallback::Sync(...)"),
            SendCallback::Async(_) => write!(f, "SendCallback::Async(...)"),
        }
    }
}


#[derive(Default)]
pub enum SendSuccess {
    /// Indicates that the data was sent successfully without any immediate received data.
    #[default]
    Sent,
    /// Indicates that the data was sent successfully and includes data received
    /// from the remote side (possibly in response to the sent data).
    SentWithNewIncomingData(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct SendFailure (pub DXBBlock);

impl SendCallback {
    pub fn new_sync(
        f: impl Fn(DXBBlock) -> Result<SendSuccess, SendFailure> + 'static + Send + Sync,
    ) -> Self {
        SendCallback::Sync(Arc::new(f))
    }

    pub fn new_async<F, Fut>(f: F) -> Self
    where
        F: Fn(DXBBlock) -> Fut + Send + Sync + 'static,
        Fut: core::future::Future<Output = Result<(), SendFailure>> + Send + Sync + 'static,
    {
        SendCallback::Async(AsyncCallback::new(f))
    }
}

pub struct ComInterfaceConfiguration {
    uuid: ComInterfaceUUID,
    /// The properties of the interface instance
    pub properties: Rc<ComInterfaceProperties>,
    // TODO: docs
    pub new_sockets_iterator: NewSocketsIterator,
}

impl Debug for ComInterfaceConfiguration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ComInterfaceConfiguration")
            .field("uuid", &self.uuid)
            .field("properties", &self.properties)
            .finish()
    }
}

impl ComInterfaceConfiguration {

    /// Creates a new ComInterfaceConfiguration with the given properties and socket iterator.
    pub fn new<I>(
        properties: ComInterfaceProperties,
        new_sockets_iterator: I,
    ) -> Self
    where I: AsyncIterator<Item = Result<SocketConfiguration, ()>> + Send + 'static {
        ComInterfaceConfiguration {
            uuid: ComInterfaceUUID(UUID::new()),
            properties: Rc::new(properties),
            new_sockets_iterator: Box::pin(new_sockets_iterator),
        }
    }

    /// Creates a new ComInterfaceConfiguration with a single socket configuration.
    pub fn new_single_socket(
        properties: ComInterfaceProperties,
        socket_configuration: SocketConfiguration,
    ) -> Self {
        ComInterfaceConfiguration {
            uuid: ComInterfaceUUID(UUID::new()),
            properties: Rc::new(properties),
            new_sockets_iterator: Box::pin(async gen move {
                yield Ok(socket_configuration);
            }),
        }
    }

    pub fn uuid(&self) -> ComInterfaceUUID {
        self.uuid.clone()
    }
}

pub type InterfaceCloseAsyncCallback = AsyncCallback<(), ()>;


/// This trait can be implemented to provide a factory with a synchronous setup process
/// for a ComInterface implementation that can be registered on a ComHub.
/// The trait should be implemented for the setup data type of the interface.
/// Example:
/// ```
/// # use core::cell::RefCell;
/// # use datex_core::stdlib::rc::Rc;
/// use serde::{Deserialize, Serialize};
/// use datex_core::network::com_hub::errors::ComInterfaceCreateError;
/// use datex_core::network::com_interfaces::com_interface::ComInterfaceProxy;
/// use datex_core::network::com_interfaces::com_interface::factory::ComInterfaceSyncFactory;
/// use datex_core::network::com_interfaces::com_interface::properties::ComInterfaceProperties;
///
///
/// #[derive(Serialize, Deserialize)]
/// struct ExampleInterfaceSetupData {
///    pub example_data: String,
/// }
///
/// impl ComInterfaceSyncFactory for ExampleInterfaceSetupData {
///     fn create_interface(
///         self,
///     ) -> Result<ComInterfaceProperties, ComInterfaceCreateError> {
///         todo!("Initialize the interface here")
///     }
///
///     fn get_default_properties() -> ComInterfaceProperties {
///         ComInterfaceProperties {
///             interface_type: "example".to_string(),
///             ..Default::default()
///         }
///     }
/// }
pub trait ComInterfaceSyncFactory
where
    Self: DeserializeOwned,
{
    /// The factory method that is called from the ComHub on a registered interface
    /// to create a new instance of the interface.
    /// The setup data is passed as a ValueContainer and has to be downcasted
    fn factory(
        setup_data: ValueContainer,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError> {
        let setup_data = from_value_container::<Self>(&setup_data)
            .map_err(|_| ComInterfaceCreateError::SetupDataParseError)?;
        Self::create_interface(setup_data)
    }

    /// Create a new instance of the interface with the given setup data.
    /// If no instance could be created with the given setup data,
    /// None is returned.
    fn create_interface(
        self,
    ) -> Result<ComInterfaceConfiguration, ComInterfaceCreateError>;

    /// Get the default interface properties for the interface.
    fn get_default_properties() -> ComInterfaceProperties;
}

/// This trait can be implemented to provide a factory with an asynchronous setup process
/// for a ComInterface implementation that can be registered on a ComHub.
/// The trait should be implemented for the setup data type of the interface.
/// Example:
/// ```
/// # use core::cell::RefCell;
/// # use datex_core::stdlib::rc::Rc;
/// use serde::{Deserialize, Serialize};
/// use datex_core::network::com_hub::errors::ComInterfaceCreateError;
/// use datex_core::network::com_interfaces::com_interface::ComInterfaceProxy;
/// use datex_core::network::com_interfaces::com_interface::factory::ComInterfaceAsyncFactory;
/// use datex_core::network::com_interfaces::com_interface::properties::ComInterfaceProperties;
/// use datex_core::network::com_hub::managers::interfaces_manager::ComInterfaceAsyncFactoryResult;
///
/// #[derive(Serialize, Deserialize)]
/// struct ExampleInterfaceSetupData {
///    pub example_data: String,
/// }
/// impl ComInterfaceAsyncFactory for ExampleInterfaceSetupData {
///     fn create_interface(
///         self,
///         com_interface_proxy: ComInterfaceProxy,
///     ) -> ComInterfaceAsyncFactoryResult {
///         Box::pin(async move {
///             // Initialize the interface here asynchronously
///             todo!()
///         })
///     }
///     fn get_default_properties() -> ComInterfaceProperties {
///         ComInterfaceProperties {
///             interface_type: "example".to_string(),
///             ..Default::default()
///         }
///     }
/// }
pub trait ComInterfaceAsyncFactory
where
    Self: DeserializeOwned,
{
    /// The factory method that is called from the ComHub on a registered interface
    /// to create a new instance of the interface.
    /// The setup data is passed as a ValueContainer and has to be downcasted
    fn factory(
        setup_data: ValueContainer,
    ) -> ComInterfaceAsyncFactoryResult {
        Box::pin(async move {
            let setup_data = from_value_container::<Self>(&setup_data)
                .map_err(|_| ComInterfaceCreateError::SetupDataParseError)?;
            Self::create_interface(setup_data).await
        })
    }

    /// Create a new instance of the interface with the given setup data.
    /// If no instance could be created with the given setup data,
    /// None is returned.
    fn create_interface(
        self,
    ) -> ComInterfaceAsyncFactoryResult;

    /// Get the default interface properties for the interface.
    fn get_default_properties() -> ComInterfaceProperties;
}
