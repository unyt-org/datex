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
    values::{value::Value, value_container::ValueContainer},
};
use alloc::rc::Rc;
use core::{
    cell::{Ref, RefCell, RefMut},
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};
impl Apply for SharedContainer {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.base_shared_container().apply(args)
    }

    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        self.base_shared_container().apply_single(arg)
    }
}
