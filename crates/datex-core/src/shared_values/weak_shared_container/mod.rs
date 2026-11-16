use crate::shared_values::{
    ReferenceMutability, ReferencedSharedContainer, SharedContainerInner,
    SharedContainerMutability,
};
use alloc::rc::Weak;
use core::cell::RefCell;

use crate::{
    prelude::*,
    shared_values::base_shared_value_container::observers::ObserverData,
};

/// Wrapper struct for a reference to a shared value (i.e. `'shared X` or `'mut shared X`).
///
/// The inner value can either be a [SharedContainerInner::EndpointOwned] or [SharedContainerInner::External]
/// In contrast to [ReferencedSharedContainer], this stores the inner [SharedContainerInner] in a [Weak] reference,
/// which allows for the inner value to be dropped when there are no more strong references to it.
#[derive(Debug, Clone)]
pub struct WeakSharedContainer {
    /// The inner container contains the actual value which can be shared between multiple owners.
    /// This can either be a [SharedContainerInner::EndpointOwned] or a [SharedContainerInner::External]
    pub(super) inner: Weak<RefCell<SharedContainerInner>>,
    /// The mutability of the reference (either `'mut shared X` or `'shared X`)
    pub(super) reference_mutability: ReferenceMutability,
    pub(super) container_mutability: SharedContainerMutability,
    /// Field used internally to indicate that this reference should be treated as a move in the context of the compiler
    pub(super) move_indicator: bool,
    pub(super) observer_data: Rc<RefCell<ObserverData>>,
}

impl WeakSharedContainer {
    /// Upgrades the weak reference to a strong reference,
    /// returning a [ReferencedSharedContainer] if the inner value is still alive.
    pub fn upgrade(&self) -> Option<ReferencedSharedContainer> {
        self.inner.upgrade().map(|inner| ReferencedSharedContainer {
            inner,
            reference_mutability: self.reference_mutability,
            container_mutability: self.container_mutability,
            move_indicator: self.move_indicator,
            observer_data: self.observer_data.clone(),
            is_uninitialized: false,
        })
    }
}
