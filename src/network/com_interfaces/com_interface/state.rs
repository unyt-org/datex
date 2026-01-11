use crate::stdlib::sync::Arc;

use tokio::sync::Notify;

use crate::{
    network::com_interfaces::com_interface::ComInterfaceEvent,
    task::UnboundedSender,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::EnumIs)]
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
    event_sender: UnboundedSender<ComInterfaceEvent>,
    shutdown_signal: Arc<Notify>, // FIXME deprecate tokio::sync::Notify in favor of stdlib::sync::Notify
}

/// Wrapper around ComInterfaceState that sends events on state changes
impl ComInterfaceStateWrapper {
    pub fn new(
        state: ComInterfaceState,
        event_sender: UnboundedSender<ComInterfaceEvent>,
    ) -> Self {
        ComInterfaceStateWrapper {
            state,
            event_sender,
            shutdown_signal: Arc::new(Notify::new()),
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
            ComInterfaceState::NotConnected => ComInterfaceEvent::NotConnected,
            ComInterfaceState::Connected => ComInterfaceEvent::Connected,
            ComInterfaceState::Destroyed => {
                self.shutdown_signal.notify_waiters();
                ComInterfaceEvent::Destroyed
            }
            ComInterfaceState::Closing | ComInterfaceState::Connecting => {
                return;
            } // No event for connecting state
        };
        let _ = self.event_sender.start_send(event);
    }

    pub fn shutdown_signal(&self) -> Arc<Notify> {
        self.shutdown_signal.clone()
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
