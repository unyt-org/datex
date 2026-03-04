use crate::network::{
    com_hub::ComHub,
    com_interfaces::com_interface::socket::ComInterfaceSocketUUID,
};

impl ComHub {
    pub async fn remove_socket(
        &self,
        socket_uuid: ComInterfaceSocketUUID,
    ) -> Result<(), ()> {
        self.socket_manager.remove_socket(socket_uuid).await
    }
}
