//! This module contains the implementation of the [ValueContainer] enum, which represents a container for values in the DATEX type system.
//! A [ValueContainer] can either be a local value, which directly contains a [Value], or a shared value, which contains a reference to a [SharedContainer].
use crate::{
    traits::{identity::Identity, structural_eq::StructuralEq},
    values::value_container::value_key::BorrowedValueKey,
};
pub mod equality;
pub mod identity;
use core::result::Result;
pub mod serde_dif;
use super::value::Value;
use crate::{
    prelude::*,
    runtime::memory::Memory,
    serde::{
        deserializer::{DatexDeserializer, from_value_container},
        error::{DeserializationError, SerializationError},
        serializer::to_value_container,
    },
    shared_values::{SharedContainer, errors::AccessError},
    traits::{apply::Apply, value_eq::ValueEq},
    types::{
        r#type::Type,
        type_definition::TypeDefinition,
        type_definition_with_metadata::{
            TypeDefinitionWithMetadata, TypeMetadata,
        },
    },
    value_updates::update_handler::UpdateHandler,
    values::core_value::CoreValue,
};

pub mod apply;
pub mod ops;
pub mod update_handler;
pub mod value_key;
use core::{
    fmt::Display,
    hash::{Hash, Hasher},
    ops::{Add, FnOnce, Neg, Sub},
};
pub mod error;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

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

    /// Tries to get the current collapsed value as a specific [CoreValue] variant.
    /// Does not perform any type conversion.
    /// Note: this performs a clone on the collapsed value
    pub fn try_as<T>(&self) -> Option<T>
    where
        T: TryFrom<CoreValue>,
    {
        self.with_collapsed_value(|value| value.inner.clone().try_as())
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
    pub fn actual_type(&self, memory: &Memory) -> Type {
        match self {
            ValueContainer::Local(local) => local.actual_type(memory).clone(),
            ValueContainer::Shared(shared) => {
                shared.actual_type(memory).clone()
            }
        }
    }

    /// Returns the actual type that describes the value container (e.g. integer or 'mut shared mut integer).
    pub fn actual_container_type(&self, memory: &Memory) -> Type {
        match self {
            ValueContainer::Local(value) => value.actual_type(memory),
            ValueContainer::Shared(shared) => {
                let inner_type =
                    shared.value_container().actual_container_type(memory);
                Type::Alias(TypeDefinitionWithMetadata {
                    definition: TypeDefinition::Nested(Box::new(inner_type)),
                    metadata: TypeMetadata::Shared {
                        mutability: shared.container_mutability(),
                        ownership: shared.ownership(),
                    },
                })
            }
        }
    }

    /// Returns the allowed type of the value container
    /// For local values, this is the same as the actual type.
    /// For shared values, this is the defined allowed type
    pub fn allowed_type(&self, memory: &Memory) -> Type {
        match self {
            ValueContainer::Local(value) => value.actual_type(memory),
            ValueContainer::Shared(shared) => shared.allowed_type().clone(),
        }
    }

    /// Casts the contained Value or Reference to the desired type T using serde deserialization.
    pub fn cast_to_deserializable<T: DeserializeOwned>(
        &self,
    ) -> Result<T, DeserializationError> {
        from_value_container::<T>(self)
    }

    /// Creates a ValueContainer from a serializable value T using serde serialization.
    pub fn from_serializable<T: serde::Serialize>(
        value: &T,
    ) -> Result<ValueContainer, SerializationError> {
        to_value_container(value)
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

impl<'a> Deserialize<'a> for ValueContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        // IMPORTANT: this only works if deserializer is actually a DatexDeserializer
        let deserializer: &DatexDeserializer = unsafe {
            &*(&deserializer as *const D as *const DatexDeserializer)
        };

        Ok(deserializer.to_value_container().into_owned())
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
            ValueContainer::Shared(reference) => reference
                .with_collapsed_value_mut(|reference| {
                    write!(f, "&({})", reference)
                }),
        }
    }
}
