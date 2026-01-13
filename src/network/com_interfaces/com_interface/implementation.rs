use crate::{
    network::{
        com_hub::{ComHub, errors::InterfaceCreateError},
        com_interfaces::com_interface::properties::InterfaceProperties,
    },
    serde::{Deserialize, deserializer::from_value_container},
    stdlib::{rc::Rc},
    values::value_container::ValueContainer,
};
pub(crate) use crate::network::com_hub::managers::interface_manager::ComInterfaceAsyncFactoryResult;
use crate::network::com_interfaces::com_interface::ComInterfaceProxy;

/// This trait can be implemented to provide a factory with a synchronous setup process 
/// for a ComInterface implementation that can be registered on a ComHub.
/// The trait should be implemented for the setup data type of the interface.
/// Example:
/// ```
/// # use core::cell::RefCell;
/// # use datex_core::stdlib::rc::Rc;
/// use serde::{Deserialize, Serialize};
/// use datex_core::network::com_hub::errors::InterfaceCreateError;
/// use datex_core::network::com_interfaces::com_interface::ComInterfaceProxy;
/// use datex_core::network::com_interfaces::com_interface::implementation::ComInterfaceSyncFactory;
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
///         com_interface_proxy: ComInterfaceProxy,
///     ) -> Result<InterfaceProperties, InterfaceCreateError> {
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
    Self: Deserialize<'static> + 'static,
{
    /// The factory method that is called from the ComHub on a registered interface
    /// to create a new instance of the interface.
    /// The setup data is passed as a ValueContainer and has to be downcasted
    fn factory(
        setup_data: ValueContainer,
        com_interface_proxy: ComInterfaceProxy,
    ) -> Result<
        InterfaceProperties,
        InterfaceCreateError,
    > {
        let setup_data = from_value_container::<Self>(setup_data)
            .map_err(|_| InterfaceCreateError::SetupDataParseError)?;
        Self::create_interface(setup_data, com_interface_proxy)
    }

    /// Register the interface on which the factory is implemented
    /// on the given ComHub.
    fn register_on_com_hub(com_hub: Rc<ComHub>) {
        let interface_type = Self::get_default_properties().interface_type;
        com_hub.register_sync_interface_factory(interface_type, Self::factory);
    }

    /// Create a new instance of the interface with the given setup data.
    /// If no instance could be created with the given setup data,
    /// None is returned.
    fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> Result<InterfaceProperties, InterfaceCreateError>;

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
/// use datex_core::network::com_hub::errors::InterfaceCreateError;
/// use datex_core::network::com_interfaces::com_interface::ComInterfaceProxy;
/// use datex_core::network::com_interfaces::com_interface::implementation::ComInterfaceAsyncFactory;
/// use datex_core::network::com_interfaces::com_interface::properties::InterfaceProperties;
/// use datex_core::network::com_hub::managers::interface_manager::ComInterfaceAsyncFactoryResult
/// use core::pin::Pin;
/// use core::future::Future;
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
    Self: Deserialize<'static> + 'static,
{
    /// The factory method that is called from the ComHub on a registered interface
    /// to create a new instance of the interface.
    /// The setup data is passed as a ValueContainer and has to be downcasted
    fn factory(
        setup_data: ValueContainer,
        com_interface_proxy: ComInterfaceProxy,
    ) -> ComInterfaceAsyncFactoryResult {
        Box::pin(async move {
            let setup_data =
                from_value_container::<Self>(setup_data)
                    .map_err(|_| InterfaceCreateError::SetupDataParseError)?;
            Self::create_interface(setup_data, com_interface_proxy).await
        })
    }

    /// Register the interface on which the factory is implemented
    /// on the given ComHub.
    fn register_on_com_hub(com_hub: Rc<ComHub>) {
        let interface_type = Self::get_default_properties().interface_type;
        com_hub.register_async_interface_factory(interface_type, Self::factory);
    }

    /// Create a new instance of the interface with the given setup data.
    /// If no instance could be created with the given setup data,
    /// None is returned.
    fn create_interface(
        self,
        com_interface_proxy: ComInterfaceProxy,
    ) -> ComInterfaceAsyncFactoryResult;

    /// Get the default interface properties for the interface.
    fn get_default_properties() -> InterfaceProperties;
}
