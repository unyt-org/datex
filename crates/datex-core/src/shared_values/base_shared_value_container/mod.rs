//! This module contains the implementation of the [BaseSharedValueContainer], which is the underlying data structure for shared values in DATEX.
use crate::{
    traits::value_eq::ValueEq,
    types::type_definition::TypeDefinition,
    utils::freemap::FreeHashMap,
    values::{
        core_value::CoreValue,
        value::Value,
        value_container::{ValueContainer, value_key::BorrowedValueKey},
    },
};
pub mod update_handler;
use crate::{
    shared_values::{
        SharedContainerMutability,
        errors::{AccessError, SharedValueCreationError},
    },
    value_updates::errors::UpdateError,
};
pub mod apply;
pub mod observers;
pub mod serde_dif;

use core::{
    fmt::{Debug, Display},
    prelude::rust_2024::*,
};
use observers::{Observer, ObserverId};

/// For the internal implementation of shared containers.
/// A BaseSharedValueContainer can only exists, when it's inner value matches the allowed type.
/// For internal modification of the value, we must ensure that the allowed type is not violated.
pub struct BaseSharedValueContainer {
    /// The value of the container
    value_container: ValueContainer,
    /// The [TypeDefinition] that is allowed to be assigned to the shared container. This is used for type checking when assigning a new value container to the shared container.
    allowed_type: TypeDefinition,
    /// List of observer callbacks
    /// TODO: move observers to ValueContainer?
    observers: FreeHashMap<ObserverId, Observer>,
    mutability: SharedContainerMutability,
}

impl BaseSharedValueContainer {
    /// Returns a new [BaseSharedValueContainer] with a [ValueContainer] containing a [CoreValue::Null], an allowed type of [CoreLibTypeId::Base(CoreLibBaseTypeId::Null)] and a mutability of [SharedContainerMutability::Immutable].
    pub fn null() -> Self {
        Self::new_with_inferred_allowed_type(
            ValueContainer::Local(Value::new(CoreValue::Null, None)),
            SharedContainerMutability::Immutable,
        )
    }

    /// Tries to create a new [BaseSharedValueContainer] with an initial [ValueContainer],
    /// an allowed type and a [SharedContainerMutability].
    /// If the allowed [TypeDefinition] is not a superset of the [ValueContainer]'s allowed type,
    /// an error is returned
    pub fn try_new(
        value_container: ValueContainer,
        allowed_type: TypeDefinition,
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
    ) -> Self {
        let value_container = value_container.into();
        let allowed_type =
            value_container.allowed_or_actual_type().into_owned();
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

    pub fn try_get_property<'a>(
        &self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, AccessError> {
        self.with_collapsed_value(|value| value.try_get_property(key))
    }

    pub fn value_container(&self) -> &ValueContainer {
        &self.value_container
    }
    pub(crate) fn value_container_mut(&mut self) -> &mut ValueContainer {
        &mut self.value_container
    }
    pub fn allowed_type(&self) -> &TypeDefinition {
        &self.allowed_type
    }
    pub fn mutability(&self) -> &SharedContainerMutability {
        &self.mutability
    }
}

impl Debug for BaseSharedValueContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BaseSharedValueContainer")
            .field("value_container", &self.value_container)
            .field("allowed_type", &self.allowed_type)
            .field("mutability", &self.mutability)
            .field("observers", &self.observers.len())
            .finish()
    }
}

impl Display for BaseSharedValueContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "shared {}{}",
            match &self.mutability {
                SharedContainerMutability::Mutable => "mut ",
                SharedContainerMutability::Immutable => "",
            },
            self.value_container,
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
