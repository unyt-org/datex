use crate::channel::futures_intrusive::ManualResetEvent;
use std::sync::Arc;

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

#[derive(Debug)]
pub struct ComInterfaceStateWrapper {
    state: ComInterfaceState,
    event_sender: UnboundedSender<ComInterfaceStateEvent>,
    shutdown_channel: Arc<ManualResetEvent>,
}

/// Wrapper around ComInterfaceState that sends events on state changes
impl ComInterfaceStateWrapper {
    pub fn new(
        state: ComInterfaceState,
        event_sender: UnboundedSender<ComInterfaceStateEvent>,
    ) -> Self {
        ComInterfaceStateWrapper {
            state,
            event_sender,
            shutdown_channel: Arc::new(ManualResetEvent::new(false)),
        }
    }

    /// Get the current state
    pub fn get(&self) -> ComInterfaceState {
        self.state
    }

    /// Set a new state and send the corresponding event
    pub fn set(&mut self, new_state: ComInterfaceState) {
        self.state = new_state;
        let event = match new_state {
            ComInterfaceState::NotConnected => {
                ComInterfaceStateEvent::NotConnected
            }
            ComInterfaceState::Connected => ComInterfaceStateEvent::Connected,
            ComInterfaceState::Destroyed => {
                self.shutdown_channel.set();
                ComInterfaceStateEvent::Destroyed
            }
            ComInterfaceState::Closing | ComInterfaceState::Connecting => {
                return;
            } // No event for connecting state
        };
        let _ = self.event_sender.start_send(event);
    }

    pub fn shutdown_receiver(&self) -> Arc<ManualResetEvent> {
        self.shutdown_channel.clone()
    }
}

impl ComInterfaceState {
    pub fn is_destroyed_or_not_connected(&self) -> bool {
        core::matches!(
            self,
            ComInterfaceState::Destroyed | ComInterfaceState::NotConnected
        )
    }
}
