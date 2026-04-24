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

impl Eq for SharedContainer {}

/// PartialEq corresponds to pointer equality / identity for `Reference`.
impl PartialEq for SharedContainer {
    fn eq(&self, other: &Self) -> bool {
        self.identical(other)
    }
}

impl StructuralEq for SharedContainer {
    fn structural_eq(&self, other: &Self) -> bool {
        self.inner()
            .base_shared_container()
            .value_container
            .structural_eq(
                &other.inner().base_shared_container().value_container,
            )
    }
}

impl ValueEq for SharedContainer {
    fn value_eq(&self, other: &Self) -> bool {
        self.inner()
            .base_shared_container()
            .value_container
            .value_eq(&other.inner().base_shared_container().value_container)
    }
}
