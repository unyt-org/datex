use crate::network::com_hub::ComHub;

#[cfg(feature = "native_tcp")]
pub mod tcp;
#[cfg(feature = "native_websocket")]
pub mod websocket;
#[cfg(feature = "native_http")]
pub mod http;
#[cfg(feature = "native_serial")]
pub mod serial;
#[cfg(feature = "native_webrtc")]
pub mod webrtc;


/// Registers all enabled native interface factories to the provided ComHub.
pub fn register_native_interface_factories(com_hub: &ComHub) {
    #[cfg(feature = "native_websocket")]
    {
        com_hub.register_async_interface_factory::<websocket::websocket_client::WebSocketClientInterfaceSetupDataNative>();
        com_hub.register_async_interface_factory::<websocket::websocket_server::WebSocketServerInterfaceSetupDataNative>();
    }
    #[cfg(feature = "native_serial")]
    {
        com_hub.register_sync_interface_factory::<serial::serial_client::SerialClientInterfaceSetupDataNative>();
    }
    #[cfg(feature = "native_tcp")]
    {
        com_hub.register_async_interface_factory::<tcp::tcp_client::TCPClientInterfaceSetupDataNative>();
        com_hub.register_async_interface_factory::<tcp::tcp_server::TCPServerInterfaceSetupDataNative>();
    }
    #[cfg(feature = "native_http")]
    {
        com_hub.register_async_interface_factory::<http::http_server::HTTPServerInterfaceSetupDataNative>();
    }
    // TODO:
    // #[cfg(feature = "native_webrtc")]
}