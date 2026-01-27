use crate::{
    channel::mpsc::UnboundedSender,
    global::dxb_block::DXBBlock,
    network::com_interfaces::com_interface::{
        ComInterfaceUUID,
        properties::InterfaceDirection,
        socket::{
            ComInterfaceSocketUUID,
        },
    },
    runtime::AsyncContext,
    values::core_values::endpoint::Endpoint,
};
use crate::stdlib::vec::Vec;

#[derive(Debug)]
pub struct ComInterfaceSocketManager {
    interface_uuid: ComInterfaceUUID,
    socket_event_sender: UnboundedSender<ComInterfaceSocketEvent>,
    async_context: AsyncContext,
}

impl ComInterfaceSocketManager {
    pub fn new_with_sender(
        interface_uuid: ComInterfaceUUID,
        sender: UnboundedSender<ComInterfaceSocketEvent>,
        async_context: AsyncContext,
    ) -> Self {
        ComInterfaceSocketManager {
            interface_uuid,
            socket_event_sender: sender,
            async_context,
        }
    }
}

impl ComInterfaceSocketManager {
    /// Adds a new socket with the Open state and notifies listeners on ComHub
    pub fn add_socket(&mut self, socket: ComInterfaceSocket) {
        self.socket_event_sender
            .start_send(ComInterfaceSocketEvent::NewSocket(socket))
            .unwrap();
    }

    /// Removes a socket by its UUID and notifies listeners on ComHub
    pub fn remove_socket(&mut self, socket_uuid: ComInterfaceSocketUUID) {
        self.socket_event_sender
            .start_send(ComInterfaceSocketEvent::CloseSocket(socket_uuid, None))
            .unwrap();
        // FIXME socket state (socket should no longer exist)
    }

    /// Removes a socket by its UUID and notifies listeners on ComHub
    pub fn remove_socket_with_unsent_block(
        &mut self,
        socket_uuid: ComInterfaceSocketUUID,
        unsent_block: DXBBlock,
    ) {
        self.socket_event_sender
            .start_send(ComInterfaceSocketEvent::CloseSocket(
                socket_uuid,
                Some(unsent_block),
            ))
            .unwrap();
        // FIXME socket state (socket should no longer exist)
    }

    /// Creates and initializes a new socket and returns its UUID and sender
    pub fn create_and_init_socket_with_optional_endpoint(
        &mut self,
        direction: InterfaceDirection,
        channel_factor: u32,
        direct_endpoint: Option<Endpoint>,
    ) -> (ComInterfaceSocketUUID, UnboundedSender<Vec<u8>>) {
        let (socket, sender) = ComInterfaceSocket::init(
            self.interface_uuid.clone(),
            direction,
            channel_factor,
            direct_endpoint,
            &self.async_context,
        );
        let socket_uuid = socket.uuid.clone();
        self.add_socket(socket);
        (socket_uuid, sender)
    }
}
