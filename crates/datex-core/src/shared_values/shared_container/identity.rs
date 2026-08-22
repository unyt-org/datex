use alloc::rc::Rc;

use crate::{
    shared_values::{SharedContainer, internal_traits::_ExposeRcInternal},
    traits::identity::Identity,
};
/// Two references are identical if they point to the same inner value (Rc pointer equality)
impl Identity for SharedContainer {
    fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(self.get_rc_internal(), other.get_rc_internal())
    }
}
