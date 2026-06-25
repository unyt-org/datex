use alloc::rc::Rc;
use core::hash::{Hash, Hasher};
use crate::{
    shared_values::SharedContainer,
    traits::{
        identity::Identity, structural_eq::StructuralEq, value_eq::ValueEq,
    },
};
use crate::shared_values::internal_traits::_ExposeRcInternal;

impl Eq for SharedContainer {}

/// PartialEq corresponds to pointer equality / identity for `Reference`.
impl PartialEq for SharedContainer {
    fn eq(&self, other: &Self) -> bool {
        self.identical(other)
    }
}

impl Hash for SharedContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(self.get_rc_internal());
        ptr.hash(state); // hash the address
    }
}

impl StructuralEq for SharedContainer {
    fn structural_eq(&self, other: &Self) -> bool {
        self.inner()
            .base_shared_container()
            .value_container()
            .structural_eq(
                other.inner().base_shared_container().value_container(),
            )
    }
}

impl ValueEq for SharedContainer {
    fn value_eq(&self, other: &Self) -> bool {
        self.inner()
            .base_shared_container()
            .value_container()
            .value_eq(other.inner().base_shared_container().value_container())
    }
}
