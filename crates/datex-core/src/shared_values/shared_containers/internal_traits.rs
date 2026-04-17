use alloc::rc::Rc;
use core::cell::RefCell;
use crate::shared_values::pointer_address::ExternalPointerAddress;
use crate::shared_values::shared_containers::{ExternalSharedContainer, SharedContainer};
use crate::shared_values::shared_containers::base_shared_value_container::BaseSharedValueContainer;

/// Internally used trait that exposed an [Rc]
/// This is only used inside [SharedContainer] for identity comparison and hashing 
/// Do not expose publicly.
pub(crate) trait _ExposeRcInternal {
    type Shared;
    fn get_rc_internal(&self) -> &Rc<RefCell<Self::Shared>>;
}