use std::async_iter::AsyncIterator;
use std::pin::Pin;
pub(crate) use crate::network::com_hub::managers::interfaces_manager::ComInterfaceAsyncFactoryResult;
use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            properties::InterfaceProperties,
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
use crate::utils::uuid::UUID;
use crate::values::core_values::endpoint::Endpoint;

pub type NewSocketsIterator = Pin<Box<dyn AsyncIterator<Item = Result<SocketConfiguration, ()>> + Send>>;

pub struct SocketProperties {
    pub direction: InterfaceDirection,
    pub channel_factor: u16,
    pub endpoint: Option<Endpoint>,
    uuid: ComInterfaceSocketUUID,
}

impl SocketProperties {
    pub fn new(
        direction: InterfaceDirection,
        channel_factor: u16,
    ) -> Self {
        SocketProperties {
            direction,
            channel_factor,
            endpoint: None,
            uuid: ComInterfaceSocketUUID(UUID::new()),
        }
    }
    pub fn new_with_endpoint(
        direction: InterfaceDirection,
        channel_factor: u16,
        endpoint: Endpoint,
    ) -> Self {
        SocketProperties {
            direction,
            channel_factor,
            endpoint: Some(endpoint),
            uuid: ComInterfaceSocketUUID(UUID::new()),
        }
    }

    pub fn uuid(&self) -> ComInterfaceSocketUUID {
        self.uuid.clone()
    }
}

pub type SocketDataIterator = Pin<Box<dyn AsyncIterator<Item = Result<Vec<u8>, ()>> + Send>>;

pub struct SocketConfiguration {
    properties: SocketProperties,
    /// An asynchronous iterator that yields incoming data from the socket as Vec<u8>
    /// It is driven by the com hub to receive data from the socket
    iterator: Option<SocketDataIterator>,
    /// A callback that is called by the com hub to send data through the socket
    /// This can be either a synchronous or asynchronous callback depending on the interface implementation
    send_callback: Option<SendCallback>
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
pub enum SendCallback {
    /// A synchronous send callback.
    /// The callback receives a DXBBlock and the UUID of the socket to send the data through.
    /// It returns a SendSuccess result which can contain already received data from the remote side.
    /// The failure case returns a SendFailure containing the original DXBBlock.
    Sync(Box<dyn Fn(DXBBlock) -> Result<SendSuccess, SendFailure> + 'static + Send>),
    /// An asynchronous send callback.
    /// The callback receives a DXBBlock and the UUID of the socket to send the data through.
    /// It returns a future that resolves to a Result indicating success or failure.
    /// The success case does not return any data, as any received data should be handled
    /// through the receive iterator.
    /// The failure case returns a SendFailure containing the original DXBBlock.
    Async(AsyncCallback<DXBBlock, Result<(), SendFailure>>),
}


#[derive(Default)]
pub enum SendSuccess {
    /// Indicates that the data was sent successfully without any immediate received data.
    #[default]
    Sent,
    /// Indicates that the data was sent successfully and includes data received
    /// from the remote side (possibly in response to the sent data).
    SentWithNewIncomingData(Vec<Vec<u8>>),
}

pub struct SendFailure (pub DXBBlock);

impl SendCallback {
    pub fn new_sync(
        f: impl Fn(DXBBlock) -> Result<SendSuccess, SendFailure> + 'static + Send,
    ) -> Self {
        SendCallback::Sync(Box::new(f))
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
    pub properties: InterfaceProperties,
    // TODO: docs
    pub new_sockets_iterator: NewSocketsIterator,
}

impl ComInterfaceConfiguration {
    
    /// Creates a new ComInterfaceConfiguration with the given properties and socket iterator.
    pub fn new<I>(
        properties: InterfaceProperties,
        new_sockets_iterator: I,
    ) -> Self
    where I: AsyncIterator<Item = Result<SocketConfiguration, ()>> + Send + 'static {
        ComInterfaceConfiguration {
            uuid: ComInterfaceUUID(UUID::new()),
            properties,
            new_sockets_iterator: Box::pin(new_sockets_iterator),
        }
    }
    
    /// Creates a new ComInterfaceConfiguration with a single socket configuration.
    pub fn new_single_socket(
        properties: InterfaceProperties,
        socket_configuration: SocketConfiguration,
    ) -> Self {
        ComInterfaceConfiguration {
            uuid: ComInterfaceUUID(UUID::new()),
            properties,
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
/// use datex_core::network::com_interfaces::com_interface::properties::InterfaceProperties;
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
///     ) -> Result<InterfaceProperties, ComInterfaceCreateError> {
///         todo!("Initialize the interface here")
///     }
///
///     fn get_default_properties() -> InterfaceProperties {
///         InterfaceProperties {
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
    fn get_default_properties() -> InterfaceProperties;
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
/// use datex_core::network::com_interfaces::com_interface::properties::InterfaceProperties;
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
///     fn get_default_properties() -> InterfaceProperties {
///         InterfaceProperties {
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
    fn get_default_properties() -> InterfaceProperties;
}
