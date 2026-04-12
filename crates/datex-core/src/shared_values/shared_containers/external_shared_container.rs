use crate::shared_values::pointer_address::ExternalPointerAddress;
use crate::shared_values::shared_container::SharedContainerValueOrType;

/// A shared container with an external pointer
#[derive(Debug)]
pub struct ExternalSharedContainer {
    pub value_or_type: SharedContainerValueOrType,
    /// Address of the external pointer, can be a remote or builtin pointer address
    pub address: ExternalPointerAddress,
}
