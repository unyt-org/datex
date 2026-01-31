#[cfg(feature = "com_http")]
pub mod http;
#[cfg(feature = "com_serial")]
pub mod serial;
#[cfg(feature = "com_tcp")]
pub mod tcp;
#[cfg(feature = "com_webrtc")]
pub mod webrtc;
#[cfg(feature = "com_websocket")]
pub mod websocket;
pub mod http_common;

/// Creates a new type that wraps the given setup data type and implements Deref to it so that
/// factory traits can be implemented on it in external crates.
#[macro_export]
macro_rules! derive_setup_data {
    ($new_type:ident, $setup_data_type:ty) => {
        #[derive(serde::Deserialize, serde::Serialize)]
        pub struct $new_type(pub $setup_data_type);
        impl core::ops::Deref for $new_type {
            type Target = $setup_data_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl core::convert::From<$setup_data_type> for $new_type {
            fn from(value: $setup_data_type) -> Self {
                Self(value)
            }
        }
    };
}