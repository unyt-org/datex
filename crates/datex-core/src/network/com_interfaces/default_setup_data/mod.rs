pub mod http;
pub mod http_common;
pub mod serial;
pub mod tcp;
pub mod webrtc;
pub mod websocket;

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
