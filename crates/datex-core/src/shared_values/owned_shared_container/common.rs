use crate::{
    shared_values::{
        OwnedSharedContainer, SharedContainerInner, SharedContainerMutability,
        SharedContainerOwnership,
        base_shared_value_container::BaseSharedValueContainer,
        traits::SharedContainerCommon,
    },
    types::type_definition::TypeDefinition,
    values::value_container::ValueContainer,
};
use core::cell::{Ref, RefMut};

use crate::shared_values::base_shared_value_container::observers::ObserverData;

impl SharedContainerCommon for OwnedSharedContainer {
    /// Get the [SharedContainerMutability] of the container
    fn container_mutability(&self) -> SharedContainerMutability {
        self.container_mutability
    }
    /// Gets a [Ref] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    fn value_container(&self) -> Ref<'_, ValueContainer> {
        Ref::map(self.base_shared_container(), |base_shared_container| {
            base_shared_container.value_container()
        })
    }

    /// Gets a [RefMut] to the currently assigned [ValueContainer] of the shared container (not resolved recursively)
    fn value_container_mut(&self) -> RefMut<'_, ValueContainer> {
        RefMut::map(self.base_shared_container_mut(), |base_shared_container| {
            base_shared_container.value_container_mut()
        })
    }

    /// Gets a [Ref] to the currently assigned allowed [Type] of the shared container (not resolved recursively)
    fn allowed_type(&self) -> Ref<'_, TypeDefinition> {
        Ref::map(self.base_shared_container(), |base_shared_container| {
            base_shared_container.allowed_type()
        })
    }

    fn is_borrowed(&self) -> bool {
        self.inner.try_borrow_mut().is_err()
    }

    /// Checks if the owned container can be mutated by the local endpoint
    fn can_mutate(&self) -> bool {
        self.container_mutability() == SharedContainerMutability::Mutable
    }

    fn inner(&self) -> Ref<'_, SharedContainerInner> {
        self.inner.borrow()
    }

    fn inner_mut(&self) -> RefMut<'_, SharedContainerInner> {
        self.inner.borrow_mut()
    }

    /// Gets a [Ref] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    fn base_shared_container(&self) -> Ref<'_, BaseSharedValueContainer> {
        Ref::map(self.inner(), |inner| inner.base_shared_container())
    }

    /// Gets a [RefMut] to the currently assigned [BaseSharedValueContainer] of the shared container (not resolved recursively)
    fn base_shared_container_mut(
        &self,
    ) -> RefMut<'_, BaseSharedValueContainer> {
        RefMut::map(self.inner_mut(), |inner| inner.base_shared_container_mut())
    }

    fn ownership(&self) -> SharedContainerOwnership {
        SharedContainerOwnership::Owned
    }

    fn observer_data(&self) -> Ref<'_, ObserverData> {
        self.observer_data.borrow()
    }

    fn observer_data_mut(&self) -> RefMut<'_, ObserverData> {
        self.observer_data.borrow_mut()
    }
}
