use crate::{
    shared_values::observers::{Observer, ObserverId},
    traits::value_eq::ValueEq,
    utils::freemap::FreeHashMap,
    values::{
        value::Value,
        value_container::{ValueContainer, value_key::BorrowedValueKey},
    },
};
pub mod update_handler;
use crate::{
    runtime::memory::Memory,
    shared_values::{
        SharedContainerMutability,
        errors::{AccessError, SharedValueCreationError},
    },
    types::r#type::Type,
    value_updates::errors::UpdateError,
};
pub mod apply;
use core::{
    fmt::{Debug, Display},
    prelude::rust_2024::*,
};

pub struct BaseSharedValueContainer {
    /// The value of the container
    pub value_container: ValueContainer,
    /// The [Type] that is allowed to be assigned to the shared container. This is used for type checking when assigning a new value container to the shared container.
    pub allowed_type: Type,
    /// List of observer callbacks
    /// TODO: move observers to ValueContainer?
    pub observers: FreeHashMap<ObserverId, Observer>,
    pub mutability: SharedContainerMutability,
}

impl BaseSharedValueContainer {
    /// Tries to create a new [BaseSharedValueContainer] with an initial [ValueContainer],
    /// an allowed type and a [SharedContainerMutability].
    /// If the allowed [TypeDefinition] is not a superset of the [ValueContainer]'s allowed type,
    /// an error is returned
    pub fn try_new(
        value_container: ValueContainer,
        allowed_type: Type,
        mutability: SharedContainerMutability,
    ) -> Result<Self, SharedValueCreationError> {
        // TODO #286: make sure allowed type is superset of reference's allowed type

        Ok(BaseSharedValueContainer {
            value_container,
            allowed_type,
            observers: FreeHashMap::new(),
            mutability,
        })
    }

    /// Creates a new [BaseSharedValueContainer] with an initial [ValueContainer] and
    /// a [SharedContainerMutability].
    /// The allowed type is inferred from the value_container's allowed type.
    pub fn new_with_inferred_allowed_type<T: Into<ValueContainer>>(
        value_container: T,
        mutability: SharedContainerMutability,
        memory: &Memory,
    ) -> Self {
        let value_container = value_container.into();
        let allowed_type = value_container.allowed_type(memory);
        BaseSharedValueContainer {
            value_container,
            allowed_type,
            observers: FreeHashMap::new(),
            mutability,
        }
    }

    /// Calls the provided callback with a mut reference to the recursively collapsed inner value of the shared container
    pub fn with_collapsed_value_mut<R>(
        &mut self,
        f: impl FnOnce(&mut Value) -> R,
    ) -> R {
        match &mut self.value_container {
            ValueContainer::Local(v) => f(v),
            ValueContainer::Shared(shared) => {
                shared.with_collapsed_value_mut(f)
            }
        }
    }

    /// Calls the provided callback with a reference to the recursively collapsed inner value of the shared container
    pub fn with_collapsed_value<R>(&self, f: impl FnOnce(&Value) -> R) -> R {
        match &self.value_container {
            ValueContainer::Local(v) => f(v),
            ValueContainer::Shared(shared) => shared.with_collapsed_value(f),
        }
    }

    /// Sets the currently assigned [ValueContainer] of the shared container to a new value container.
    /// Returns the [ValueContainer] as an error if the new value container's allowed type is not compatible with the allowed type of the shared container
    pub fn try_set_value_container(
        &mut self,
        new_value_container: ValueContainer,
    ) -> Result<(), ValueContainer> {
        // TODO do type checking to ensure new value container's allowed type is compatible with self.allowed_type
        self.value_container = new_value_container;
        Ok(())
    }

    pub fn try_get_property<'a>(
        &self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, AccessError> {
        self.with_collapsed_value(|value| value.try_get_property(key))
    }
}

impl Debug for BaseSharedValueContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReferenceData")
            .field("value_container", &self.value_container)
            .field("allowed_type", &self.allowed_type)
            .field("observers", &self.observers.len())
            .finish()
    }
}

impl Display for BaseSharedValueContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "shared {}{}",
            self.value_container,
            match &self.mutability {
                SharedContainerMutability::Mutable => "mut ",
                SharedContainerMutability::Immutable => "",
            }
        )
    }
}

impl PartialEq for BaseSharedValueContainer {
    fn eq(&self, other: &Self) -> bool {
        // Two value references are equal if their value containers are equal
        self.value_container.value_eq(&other.value_container)
    }
}

impl BaseSharedValueContainer {
    pub fn current_value_container(&self) -> &ValueContainer {
        &self.value_container
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self.mutability, SharedContainerMutability::Mutable)
    }

    pub fn assert_can_mutate(&self) -> Result<(), UpdateError> {
        if !self.is_mutable() {
            return Err(UpdateError::ImmutableValue);
        }
        Ok(())
    }
}
