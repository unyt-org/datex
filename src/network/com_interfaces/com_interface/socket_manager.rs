use crate::{
    network::com_interfaces::com_interface::{
        ComInterfaceUUID,
        properties::InterfaceDirection,
        socket::{
            ComInterfaceSocket, ComInterfaceSocketEvent, ComInterfaceSocketUUID,
        },
    },
    task::UnboundedSender,
    values::core_values::endpoint::Endpoint,
};

#[derive(Debug)]
pub struct ComInterfaceSocketManager {
    interface_uuid: ComInterfaceUUID,
    socket_event_sender: UnboundedSender<ComInterfaceSocketEvent>,
}

impl ComInterfaceSocketManager {
    pub fn new_with_sender(
        interface_uuid: ComInterfaceUUID,
        sender: UnboundedSender<ComInterfaceSocketEvent>,
    ) -> Self {
        ComInterfaceSocketManager {
            interface_uuid,
            socket_event_sender: sender,
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
            .start_send(ComInterfaceSocketEvent::RemovedSocket(socket_uuid))
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
        );
        let socket_uuid = socket.uuid.clone();
        self.add_socket(socket);
        (socket_uuid, sender)
    }
}
