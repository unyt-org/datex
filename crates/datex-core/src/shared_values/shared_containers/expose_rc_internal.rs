use alloc::rc::Rc;
use core::cell::RefCell;
use crate::shared_values::shared_containers::SharedContainer;

/// Internally used trait that exposed an [Rc]
/// This is only used inside [SharedContainer] for identity comparison and hashing 
/// Do not expose publicly.
pub(crate) trait ExposeRcInternal {
    type Shared;
    fn get_rc_internal(&self) -> &Rc<RefCell<Self::Shared>>;
}