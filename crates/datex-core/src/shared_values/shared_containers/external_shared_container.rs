use crate::shared_values::pointer_address::ExternalPointerAddress;
use crate::shared_values::shared_container::shared_value_container::SharedValueContainer;

/// A shared container with an external pointer
#[derive(Debug)]
pub struct ExternalSharedContainer {
    pub value: SharedValueContainer,
    /// Address of the external pointer, can be a remote or builtin pointer address
    pub address: ExternalPointerAddress,
}
