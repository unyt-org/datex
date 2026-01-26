use std::async_iter::AsyncIterator;
use std::pin::Pin;
pub(crate) use crate::network::com_hub::managers::interfaces_manager::ComInterfaceAsyncFactoryResult;
use crate::{
    network::{
        com_hub::errors::ComInterfaceCreateError,
        com_interfaces::com_interface::{
            ComInterfaceProxy, properties::InterfaceProperties,
        },
    },
    serde::deserializer::from_value_container,
    values::value_container::ValueContainer,
};
use serde::de::DeserializeOwned;
use crate::global::dxb_block::DXBBlock;
use crate::network::com_hub::InterfacePriority;
use crate::network::com_interfaces::com_interface::properties::InterfaceDirection;
use crate::network::com_interfaces::com_interface::socket::ComInterfaceSocketUUID;
use crate::stdlib::boxed::Box;
use crate::utils::async_callback::AsyncCallback;
use crate::utils::uuid::UUID;
use crate::values::core_values::endpoint::Endpoint;

pub struct NewSocketsIterator {
    iterator: Pin<Box<dyn AsyncIterator<Item = Result<SocketDataIterator, ()>> + Send>>
}

impl NewSocketsIterator {
    pub fn new_multiple<I>(iter: I) -> Self
    where
        I: AsyncIterator<Item = Result<SocketDataIterator, ()>> + Send + 'static,
    {
        NewSocketsIterator { iterator: Box::pin(iter) }
    }
    
    /// directly returns a NewSocketsIterator from a single SocketDataIterator
    pub fn new_single(socket_iterator: SocketDataIterator) -> Self {
        NewSocketsIterator {
            iterator: Box::pin(async gen move {
                yield Ok(socket_iterator);
            }),
        }
    }
}

pub struct SocketConfiguration {
    pub direction: InterfaceDirection,
    pub channel_factor: u16,
    pub endpoint: Option<Endpoint>,
    uuid: ComInterfaceSocketUUID,
}

impl SocketConfiguration {
    pub fn new(
        direction: InterfaceDirection,
        channel_factor: u16,
    ) -> Self {
        SocketConfiguration {
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
        SocketConfiguration {
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

pub struct SocketDataIterator {
    socket_configuration: SocketConfiguration,
    iterator: Option<Pin<Box<dyn AsyncIterator<Item = Result<Vec<u8>, ()>> + Send>>>
}
impl SocketDataIterator {
    /// Creates a SocketDataIterator for a given socket with the provided parameters.
    /// The iterator parameter is an asynchronous iterator that yields incoming data from the socket as Vec<u8>.
    pub fn new<I>(
        socket_configuration: SocketConfiguration,
        iter: I,
    ) -> Self
    where
        I: AsyncIterator<Item = Result<Vec<u8>, ()>> + Send + 'static,
    {
        SocketDataIterator {
            socket_configuration,
            iterator: Some(Box::pin(iter)),
        }
    }
    
    /// Creates a SocketDataIterator without a data iterator.
    /// This can be used for interfaces that do not support incoming data,
    /// e.g., for output-only interfaces, or if all incoming data is handled in the send callback.
    pub fn new_no_iterator(
        socket_configuration: SocketConfiguration,
    ) -> Self {
        SocketDataIterator {
            socket_configuration,
            iterator: None,
        }
    }
}

impl From<SocketConfiguration> for SocketDataIterator {
    fn from(socket_configuration: SocketConfiguration) -> Self {
        SocketDataIterator::new_no_iterator(socket_configuration)
    }
}

/// A callback that is called by the com hub to send data through the interface.
pub enum SendCallback {
    /// A synchronous send callback.
    /// The callback receives a DXBBlock and the UUID of the socket to send the data through.
    /// It returns a SendSuccess result which can contain already received data from the remote side.
    /// The failure case returns a SendFailure containing the original DXBBlock.
    Sync(Box<dyn Fn((DXBBlock, ComInterfaceSocketUUID)) -> Result<SendSuccess, SendFailure>>),
    /// An asynchronous send callback.
    /// The callback receives a DXBBlock and the UUID of the socket to send the data through.
    /// It returns a future that resolves to a Result indicating success or failure.
    /// The success case does not return any data, as any received data should be handled
    /// through the receive iterator.
    /// The failure case returns a SendFailure containing the original DXBBlock.
    Async(AsyncCallback<(DXBBlock, ComInterfaceSocketUUID), Result<(), SendFailure>>),
}


#[derive(Default)]
pub enum SendSuccess {
    /// Indicates that the data was sent successfully without any immediate received data.
    #[default]
    Sent,
    /// Indicates that the data was sent successfully and includes data received
    /// from the remote side (possibly in response to the sent data).
    SentAndReceivedData(Vec<Vec<u8>>),
}

pub struct SendFailure (pub DXBBlock);

impl SendCallback {
    pub fn new_sync(
        f: impl Fn((DXBBlock, ComInterfaceSocketUUID)) -> Result<SendSuccess, SendFailure> + 'static,
    ) -> Self {
        SendCallback::Sync(Box::new(f))
    }

    pub fn new_async<F, Fut>(f: F) -> Self
    where
        F: Fn((DXBBlock, ComInterfaceSocketUUID)) -> Fut + Send + 'static,
        Fut: core::future::Future<Output = Result<(), SendFailure>> + Send + 'static,
    {
        SendCallback::Async(AsyncCallback::new(f))
    }
}

pub struct ComInterfaceConfiguration {
    /// The properties of the interface instance
    pub properties: InterfaceProperties,
    /// A callback that is called by the com hub to send data through the interface
    /// This can be either a synchronous or asynchronous callback depending on the interface implementation
    pub send_callback: SendCallback,
    /// A callback that is called by the com hub when the interface is closed
    /// If None, no special cleanup logic is required on close
    pub close_callback: Option<InterfaceCloseAsyncCallback>,
    // TODO: docs
    pub new_sockets_iterator: NewSocketsIterator,
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
