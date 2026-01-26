use crate::{
    channel::mpsc::UnboundedReceiver,
    network::{
        com_hub::{
            ComHub, InterfacePriority,
            managers::sockets_manager::SocketsManager,
        },
        com_interfaces::com_interface::{
            ComInterfaceUtils, socket::ComInterfaceSocketEvent,
        },
    },
    stdlib::{cell::RefCell, rc::Rc},
    task::spawn_with_panic_notify,
};
use crate::network::com_hub::MAX_CONCURRENT_COM_INTERFACES_EMBASSY;

impl ComHub {
    pub(crate) fn handle_interface_socket_events(
        &self,
        interface: &ComInterfaceUtils,
        socket_event_receiver: UnboundedReceiver<ComInterfaceSocketEvent>,
    ) {
        let interface_uuid = interface.uuid();
        let priority = self
            .interface_manager
            .borrow()
            .interface_priority(&interface_uuid)
            .unwrap_or_default();
        spawn_with_panic_notify(
            &self.async_context,
            handle_interface_socket_events(
                socket_event_receiver,
                self.socket_manager.clone(),
                priority,
            ),
        );
    }
}

#[cfg_attr(feature = "embassy_runtime", embassy_executor::task(pool_size = MAX_CONCURRENT_COM_INTERFACES_EMBASSY))]
async fn handle_interface_socket_events(
    mut receiver_queue: UnboundedReceiver<ComInterfaceSocketEvent>,
    socket_manager: Rc<RefCell<SocketsManager>>,
    priority: InterfacePriority,
) {
    while let Some(event) = receiver_queue.next().await {
        socket_manager
            .borrow_mut()
            .handle_socket_event(event, priority)
    }
}
