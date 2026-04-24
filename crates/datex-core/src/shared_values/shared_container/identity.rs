use alloc::rc::Rc;

use crate::{
    runtime::{
        execution::ExecutionError, memory::Memory,
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    shared_values::{
        OwnedSharedContainer, PointerAddress, ReferencedSharedContainer,
        SelfOwnedPointerAddress, SharedContainer, SharedContainerInner,
        SharedContainerMutability, SharedContainerOwnership,
        base_shared_value_container::BaseSharedValueContainer,
        errors::{
            AccessError, UnexpectedImmutableReferenceError,
            UnexpectedSharedContainerOwnershipError,
        },
        internal_traits::_ExposeRcInternal,
        observers::{Observer, ObserverError, ObserverId},
    },
    traits::{
        apply::Apply, identity::Identity, structural_eq::StructuralEq,
        value_eq::ValueEq,
    },
    types::r#type::Type,
    values::{
        value::Value,
        value_container::{ValueContainer, value_key::BorrowedValueKey},
    },
};
/// Two references are identical if they point to the same inner value (Rc pointer equality)
impl Identity for SharedContainer {
    fn identical(&self, other: &Self) -> bool {
        Rc::ptr_eq(self.get_rc_internal(), other.get_rc_internal())
    }
}
