//! This module contains the implementation of the [ValueContainer] enum, which represents a container for values in the DATEX type system.
//! A [ValueContainer] can either be a local value, which directly contains a [Value], or a shared value, which contains a reference to a [SharedContainer].
use crate::values::value_container::value_key::BorrowedValueKey;
pub mod equality;
pub mod identity;
use core::result::Result;
pub mod serde_dif;
use super::value::Value;
use crate::{
    prelude::*,
    shared_values::{SharedContainer, errors::AccessError},
    types::{
        r#type::Type,
        type_definition::TypeDefinition,
        type_definition_with_metadata::{
            TypeDefinitionWithMetadata, TypeMetadata,
        },
    },
    values::core_value::CoreValue,
};

pub mod apply;
pub mod ops;
pub mod update_handler;
pub mod value_key;
use crate::shared_values::traits::SharedContainerCommon;
use core::{
    fmt::Display,
    hash::{Hash, Hasher},
    ops::FnOnce,
};

pub mod datex_proxy;
pub mod error;

#[derive(Debug, Eq, Clone)]
pub enum ValueContainer {
    Local(Value),
    Shared(SharedContainer),
}

impl ValueContainer {
    /// Creates a new [ValueContainer::Local] from a [Value]
    pub fn local(value: impl Into<Value>) -> Self {
        ValueContainer::Local(value.into())
    }

    /// Calls a fn with a reference to the current inner collapsed value of the  container
    pub fn with_collapsed_value<R, F: FnOnce(&Value) -> R>(&self, f: F) -> R {
        match self {
            ValueContainer::Local(value) => f(value),
            ValueContainer::Shared(shared) => shared.with_collapsed_value(f),
        }
    }

    /// Calls a fn with a mutable reference to the current inner collapsed value of the container
    pub(crate) fn with_collapsed_value_mut<R, F: FnOnce(&mut Value) -> R>(
        &mut self,
        f: F,
    ) -> R {
        match self {
            ValueContainer::Local(value) => f(value),
            ValueContainer::Shared(shared) => {
                shared.with_collapsed_value_mut(f)
            }
        }
    }

    /// Gets a cloned, collapsed inner value.
    /// Use [ValueContainer::with_collapsed_value] instead whenever possible
    /// or match the [ValueContainer]
    pub fn get_cloned_value(&self) -> Value {
        self.with_collapsed_value(|value| value.clone())
    }

    /// Tries to get an immutable reference to the value as a specified type.
    /// Does not perform any type conversion.
    /// This only works for local values, not for shared values.
    pub fn try_as<'a, T>(&'a self) -> Option<&'a T>
    where
        &'a T: TryFrom<&'a CoreValue>,
    {
        match self {
            ValueContainer::Local(value) => value.inner.try_as(),
            ValueContainer::Shared(_) => None,
        }
    }

    /// Tries to get a mutable reference to the value as a specified type.
    /// Does not perform any type conversion.
    /// This only works for local values, not for shared values.
    pub fn try_as_mut<'a, T>(&'a mut self) -> Option<&'a mut T>
    where
        &'a mut T: TryFrom<&'a mut CoreValue>,
    {
        match self {
            ValueContainer::Local(value) => value.inner.try_as_mut(),
            ValueContainer::Shared(_) => None,
        }
    }

    /// Tries to get the current collapsed value as a specified type.
    /// Does not perform any type conversion.
    /// Runs the provided closure with a reference to the typed value if the conversion was successful, otherwise returns None.
    pub fn try_with<T, R, F>(&self, f: F) -> Option<R>
    where
        F: for<'a> FnOnce(&'a T) -> R,
        for<'a> &'a T: TryFrom<&'a CoreValue>,
    {
        self.with_collapsed_value(|value| value.inner.try_as().map(f))
    }

    /// Tries to get the current collapsed value as a specified type.
    /// Does not perform any type conversion.
    /// This only works for local values, not for shared values.
    pub fn try_into_value<T>(self) -> Option<T>
    where
        T: TryFrom<CoreValue>,
    {
        match self {
            ValueContainer::Local(value) => value.inner.try_into().ok(),
            ValueContainer::Shared(_) => None,
        }
    }

    /// Performs a clone used by the "clone" command
    /// Local values are just cloned normally
    /// For shared value, the inner value container is cloned (shared x -> x)
    pub fn get_cloned(&self) -> ValueContainer {
        match self {
            ValueContainer::Local(value) => {
                ValueContainer::Local(value.clone())
            }
            ValueContainer::Shared(shared) => shared.value_container().clone(),
        }
    }

    /// Returns the actual type of the contained value, resolving shared values if necessary.
    pub fn actual_type(&self) -> TypeDefinition {
        match self {
            ValueContainer::Local(local) => local.actual_type().clone(),
            ValueContainer::Shared(shared) => shared.actual_type().clone(),
        }
    }

    /// Returns the actual type that describes the value container (e.g. integer or 'mut shared mut integer).
    pub fn actual_container_type(&self) -> TypeDefinitionWithMetadata {
        match self {
            ValueContainer::Local(value) => TypeDefinitionWithMetadata {
                definition: value.actual_type(),
                metadata: TypeMetadata::default(),
                reference_name: None,
            },
            ValueContainer::Shared(shared) => {
                let inner_type =
                    shared.value_container().actual_container_type();
                TypeDefinitionWithMetadata {
                    definition: TypeDefinition::Nested(Box::new(Type::from(
                        inner_type,
                    ))),
                    metadata: TypeMetadata::Shared {
                        mutability: shared.container_mutability(),
                        ownership: shared.ownership(),
                    },
                    reference_name: None,
                }
            }
        }
    }

    /// For local values, returns the actual type of the value container
    /// For shared values, returns the allowed type of the value container
    pub fn allowed_or_actual_type(&self) -> TypeDefinition {
        match self {
            ValueContainer::Local(value) => value.actual_type(),
            ValueContainer::Shared(shared) => shared.allowed_type().clone(),
        }
    }

    /// Returns the contained SharedContainer if it is a SharedContainer, otherwise returns None.
    pub fn maybe_shared(&self) -> Option<&SharedContainer> {
        if let ValueContainer::Shared(shared) = self {
            Some(shared)
        } else {
            None
        }
    }

    /// Runs a closure with the contained SharedContainer if it is a SharedContainer, otherwise returns None.
    pub fn with_maybe_shared<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&SharedContainer) -> R,
    {
        if let ValueContainer::Shared(shared) = self {
            Some(f(shared))
        } else {
            None
        }
    }

    /// Returns a reference to the contained SharedContainer, panics if it is not a SharedContainer.
    pub fn shared_unchecked(&self) -> &SharedContainer {
        match self {
            ValueContainer::Shared(shared) => shared,
            _ => {
                core::panic!("Cannot convert ValueContainer to SharedContainer")
            }
        }
    }

    pub fn try_get_property<'a>(
        &self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, AccessError> {
        match self {
            ValueContainer::Local(value) => value.try_get_property(key),
            ValueContainer::Shared(reference) => {
                reference.try_get_property(key)
            }
        }
    }
}

impl<T: Into<Value>> From<T> for ValueContainer {
    fn from(value: T) -> Self {
        ValueContainer::Local(value.into())
    }
}

impl<'a> From<BorrowedValueKey<'a>> for ValueContainer {
    fn from(value_key: BorrowedValueKey) -> Self {
        match value_key {
            BorrowedValueKey::Text(text) => {
                ValueContainer::Local(text.into_owned().into())
            }
            BorrowedValueKey::Index(index) => {
                ValueContainer::Local(index.into())
            }
            BorrowedValueKey::Value(value_container) => {
                value_container.into_owned()
            }
        }
    }
}

impl Hash for ValueContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ValueContainer::Local(value) => value.hash(state),
            ValueContainer::Shared(pointer) => pointer.hash(state),
        }
    }
}

impl Display for ValueContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ValueContainer::Local(value) => core::write!(f, "{value}"),
            // TODO #118: only simple temporary way to distinguish between Value and Pointer
            ValueContainer::Shared(shared) => {
                if shared.is_borrowed() {
                    write!(f, "{}", shared.to_string_omit_content())
                } else {
                    shared.with_collapsed_value(|reference| {
                        write!(f, "shared ({})", reference)
                    })
                }
            }
        }
    }
}
