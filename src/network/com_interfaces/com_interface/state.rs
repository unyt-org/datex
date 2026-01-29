use crate::channel::futures_intrusive::ManualResetEvent;
use crate::stdlib::sync::Arc;

use crate::{
    channel::mpsc::UnboundedSender,
    network::com_interfaces::com_interface::ComInterfaceStateEvent,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum_macros::EnumIs, strum::Display,
)]
pub enum ComInterfaceState {
    NotConnected,
    Closing,
    Connected,
    Connecting,
    Destroyed,
}

impl ComInterfaceState {
    pub fn is_destroyed_or_not_connected(&self) -> bool {
        core::matches!(
            self,
            ComInterfaceState::Destroyed | ComInterfaceState::NotConnected
        )
    }
}
