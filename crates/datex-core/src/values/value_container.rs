use crate::traits::{identity::Identity, structural_eq::StructuralEq};
use core::{cell::RefCell, result::Result};

use super::value::Value;
use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    serde::{
        deserializer::{DatexDeserializer, from_value_container},
        error::DeserializationError,
    },
    shared_values::shared_containers::observers::TransceiverId,
    traits::{apply::Apply, value_eq::ValueEq},
    types::type_definition::TypeDefinition,
    values::core_value::CoreValue,
};

use crate::{
    serde::{error::SerializationError, serializer::to_value_container},
    shared_values::{errors::AccessError, shared_containers::SharedContainer},
    types::{r#type::Type, type_definition_with_metadata::TypeMetadata},
};
use core::{
    fmt::Display,
    hash::{Hash, Hasher},
    ops::{Add, FnOnce, Neg, Sub},
};
use serde::{Deserialize, de::DeserializeOwned, Serialize};
use crate::runtime::memory::Memory;
use crate::types::type_definition_with_metadata::TypeDefinitionWithMetadata;
use crate::value_updates::errors::UpdateError;
use crate::value_updates::update_data::{AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData, ReplaceUpdateData, SetEntryUpdateData, Update};
use crate::value_updates::update_handler::UpdateHandler;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueError {
    IsVoid,
    InvalidOperation,
    IntegerOverflow,
    TypeConversionError,
}

impl Display for ValueError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ValueError::IsVoid => core::write!(f, "Value is void"),
            ValueError::InvalidOperation => {
                core::write!(f, "Invalid operation on value")
            }
            ValueError::TypeConversionError => {
                core::write!(f, "Type conversion error")
            }
            ValueError::IntegerOverflow => {
                core::write!(f, "Integer overflow occurred")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueKey {
    Text(String),
    Index(i64),
    Value(ValueContainer),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BorrowedValueKey<'a> {
    Text(Cow<'a, str>),
    Index(i64),
    Value(Cow<'a, ValueContainer>),
}

impl<'a> From<ValueKey> for BorrowedValueKey<'a> {
    fn from(owned: ValueKey) -> Self {
        match owned {
            ValueKey::Text(text) => BorrowedValueKey::Text(Cow::Owned(text)),
            ValueKey::Index(index) => BorrowedValueKey::Index(index),
            ValueKey::Value(value_container) => {
                BorrowedValueKey::Value(Cow::Owned(value_container))
            }
        }
    }
}

impl From<BorrowedValueKey<'_>> for ValueKey {
    fn from(owned: BorrowedValueKey) -> Self {
        match owned {
            BorrowedValueKey::Text(text) => ValueKey::Text(text.into_owned()),
            BorrowedValueKey::Index(index) => ValueKey::Index(index),
            BorrowedValueKey::Value(value_container) => {
                ValueKey::Value(value_container.into_owned())
            }
        }
    }
}


impl<'a> BorrowedValueKey<'a> {
    pub fn with_value_container<R>(
        &self,
        callback: impl FnOnce(&ValueContainer) -> R,
    ) -> R {
        match self {
            BorrowedValueKey::Value(value_container) => callback(value_container),
            BorrowedValueKey::Text(text) => {
                let value_container =
                    ValueContainer::Local(text.as_ref().into());
                callback(&value_container)
            }
            BorrowedValueKey::Index(index) => {
                let value_container =
                    ValueContainer::Local(Value::from(*index));
                callback(&value_container)
            }
        }
    }
}

impl<'a> Display for BorrowedValueKey<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BorrowedValueKey::Text(text) => core::write!(f, "{}", text),
            BorrowedValueKey::Index(index) => core::write!(f, "{}", index),
            BorrowedValueKey::Value(value_container) => {
                core::write!(f, "{}", value_container)
            }
        }
    }
}

impl<'a> From<&'a String> for BorrowedValueKey<'a> {
    fn from(text: &'a String) -> Self {
        BorrowedValueKey::Text(Cow::from(text))
    }
}

impl<'a> From<&'a str> for BorrowedValueKey<'a> {
    fn from(text: &'a str) -> Self {
        BorrowedValueKey::Text(Cow::from(text))
    }
}

impl<'a> From<i64> for BorrowedValueKey<'a> {
    fn from(index: i64) -> Self {
        BorrowedValueKey::Index(index)
    }
}

impl<'a> From<u32> for BorrowedValueKey<'a> {
    fn from(index: u32) -> Self {
        BorrowedValueKey::Index(index as i64)
    }
}

impl<'a> From<i32> for BorrowedValueKey<'a> {
    fn from(index: i32) -> Self {
        BorrowedValueKey::Index(index as i64)
    }
}

impl<'a> From<&'a ValueContainer> for BorrowedValueKey<'a> {
    fn from(value_container: &'a ValueContainer) -> Self {
        BorrowedValueKey::Value(Cow::Borrowed(value_container))
    }
}

impl From<ValueContainer> for BorrowedValueKey<'_> {
    fn from(value_container: ValueContainer) -> Self {
        BorrowedValueKey::Value(Cow::Owned(value_container))
    }
}

impl<'a> From<&'a str> for ValueKey {
    fn from(text: &'a str) -> Self {
        ValueKey::Text(text.to_string())
    }
}

impl From<ValueContainer> for ValueKey {
    fn from(value_container: ValueContainer) -> Self {
        ValueKey::Value(value_container)
    }
}

impl From<i32> for ValueKey {
    fn from(index: i32) -> Self {
        ValueKey::Index(index as i64)
    }
}

impl From<i64> for ValueKey {
    fn from(index: i64) -> Self {
        ValueKey::Index(index)
    }
}

impl From<u32> for ValueKey {
    fn from(index: u32) -> Self {
        ValueKey::Index(index as i64)
    }
}

impl<'a> BorrowedValueKey<'a> {
    pub fn try_as_text(&self) -> Option<&str> {
        if let BorrowedValueKey::Text(text) = self {
            Some(text)
        } else if let BorrowedValueKey::Value(val) = self
            && let ValueContainer::Local(Value {
                inner: CoreValue::Text(text),
                ..
            }) = val.as_ref()
        {
            Some(&text.0)
        } else {
            None
        }
    }

    pub fn try_as_index(&self) -> Option<i64> {
        if let BorrowedValueKey::Index(index) = self {
            Some(*index)
        } else if let BorrowedValueKey::Value(value) = self
            && let ValueContainer::Local(Value {
                inner: CoreValue::Integer(index),
                ..
            }) = value.as_ref()
        {
            index.as_i64()
        } else if let BorrowedValueKey::Value(value) = self
            && let ValueContainer::Local(Value {
                inner: CoreValue::TypedInteger(index),
                ..
            }) = value.as_ref()
        {
            index.as_i64()
        } else {
            None
        }
    }
}

impl<'a> From<BorrowedValueKey<'a>> for ValueContainer {
    fn from(value_key: BorrowedValueKey) -> Self {
        match value_key {
            BorrowedValueKey::Text(text) => {
                ValueContainer::Local(text.into_owned().into())
            }
            BorrowedValueKey::Index(index) => ValueContainer::Local(index.into()),
            BorrowedValueKey::Value(value_container) => value_container.into_owned(),
        }
    }
}

#[derive(Debug, Eq, Clone)]
pub enum ValueContainer {
    Local(Value),
    Shared(SharedContainer),
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

/// Partial equality for ValueContainer is identical to Hash behavior:
/// Identical references are partially equal, value-equal values are also partially equal.
/// A pointer and a value are never partially equal.
impl PartialEq for ValueContainer {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueContainer::Local(a), ValueContainer::Local(b)) => a == b,
            (ValueContainer::Shared(a), ValueContainer::Shared(b)) => a == b,
            _ => false,
        }
    }
}

/// Structural equality checks the structural equality of the underlying values, collapsing
/// references to their current resolved values.
impl StructuralEq for ValueContainer {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueContainer::Local(a), ValueContainer::Local(b)) => {
                a.structural_eq(b)
            }
            (ValueContainer::Shared(a), ValueContainer::Shared(b)) => {
                a.structural_eq(b)
            }
            (ValueContainer::Local(a), ValueContainer::Shared(b))
            | (ValueContainer::Shared(b), ValueContainer::Local(a)) => {
                b.with_collapsed_value_mut(|b| a.structural_eq(b))
            }
        }
    }
}

/// Value equality checks the value equality of the underlying values, collapsing
/// references to their current resolved values.
impl ValueEq for ValueContainer {
    fn value_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueContainer::Local(a), ValueContainer::Local(b)) => {
                a.value_eq(b)
            }
            (ValueContainer::Shared(a), ValueContainer::Shared(b)) => {
                a.value_eq(b)
            }
            (ValueContainer::Local(a), ValueContainer::Shared(b))
            | (ValueContainer::Shared(b), ValueContainer::Local(a)) => {
                b.with_collapsed_value_mut(|b| a.value_eq(b))
            }
        }
    }
}

/// Identity checks only returns true if two references are identical.
/// Values are never identical to references or other values.
impl Identity for ValueContainer {
    fn identical(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueContainer::Local(_), ValueContainer::Local(_)) => false,
            (ValueContainer::Shared(a), ValueContainer::Shared(b)) => {
                a.identical(b)
            }
            _ => false,
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

impl ValueContainer {
    /// Creates a new [ValueContainer::Local] from a [Value]
    pub fn local(value: impl Into<Value>) -> Self {
        ValueContainer::Local(value.into())
    }

    /// Calls a fn with a reference to the current inner collapsed value of the  container
    pub fn with_collapsed_value<R, F: FnOnce(&Value) -> R>(
        &self,
        f: F,
    ) -> R {
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

    /// Gets a cloned, collapsed inner value. Use [`ValueContainer::with_collapsed_value`] instead whenever possible
    #[deprecated(note = "use with_collapsed_value")]
    pub fn to_cloned_value(&self) -> Rc<RefCell<Value>> {
        unimplemented!("use with_collapsed_value")
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
            ValueContainer::Shared(shared) => shared.actual_type(memory).clone(),
        }
    }

    /// Returns the actual type that describes the value container (e.g. integer or 'mut shared mut integer).
    pub fn actual_container_type(&self, memory: &Memory) -> Type {
        match self {
            ValueContainer::Local(value) => {
                value.actual_type(memory)
            }
            ValueContainer::Shared(shared) => {
                let inner_type =
                    shared.value_container().actual_container_type(memory);
                Type::Alias(TypeDefinitionWithMetadata {
                    definition: TypeDefinition::Nested(Box::new(inner_type)),
                    metadata: TypeMetadata::Shared {
                        mutability: shared.container_mutability(),
                        ownership: shared.ownership(),
                    }
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

impl UpdateHandler for ValueContainer {
    fn try_replace(&self, data: ReplaceUpdateData, source_id: TransceiverId) -> Result<ValueContainer, UpdateError> {
        todo!()
    }

    fn try_set_entry(&self, data: SetEntryUpdateData, source_id: TransceiverId) -> Result<(), UpdateError> {
        todo!()
    }

    fn try_delete_entry(&self, data: DeleteEntryUpdateData, source_id: TransceiverId) -> Result<ValueContainer, UpdateError> {
        todo!()
    }

    fn try_append_entry(&self, data: AppendEntryUpdateData, source_id: TransceiverId) -> Result<(), UpdateError> {
        todo!()
    }

    fn try_clear(&self, source_id: TransceiverId) -> Result<Vec<ValueContainer>, UpdateError> {
        todo!()
    }

    fn try_list_splice(&self, data: ListSpliceUpdateData, source_id: TransceiverId) -> Result<Vec<ValueContainer>, UpdateError> {
        todo!()
    }
}

impl Apply for ValueContainer {
    fn apply(
        &self,
        args: &[ValueContainer],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match self {
            ValueContainer::Local(value) => value.apply(args),
            ValueContainer::Shared(reference) => reference.apply(args),
        }
    }

    fn apply_single(
        &self,
        arg: &ValueContainer,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        match self {
            ValueContainer::Local(value) => value.apply_single(arg),
            ValueContainer::Shared(reference) => reference.apply_single(arg),
        }
    }
}

impl<T: Into<Value>> From<T> for ValueContainer {
    fn from(value: T) -> Self {
        ValueContainer::Local(value.into())
    }
}

impl Add<ValueContainer> for ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    fn add(self, rhs: ValueContainer) -> Self::Output {
        (&self).add(&rhs)
    }
}

impl Add<&ValueContainer> for &ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    // FIXME: remove clones
    fn add(self, rhs: &ValueContainer) -> Self::Output {
        match (self, rhs) {
            (ValueContainer::Local(lhs), ValueContainer::Local(rhs)) => {
                lhs + rhs
            }
            (ValueContainer::Shared(lhs), ValueContainer::Shared(rhs)) => lhs
                .with_collapsed_value_mut(|lhs| {
                    rhs.with_collapsed_value_mut(|rhs| lhs.clone() + rhs.clone())
                }),
            (ValueContainer::Local(lhs), ValueContainer::Shared(rhs)) => {
                rhs.with_collapsed_value_mut(|rhs| lhs + rhs)
            }
            (ValueContainer::Shared(lhs), ValueContainer::Local(rhs)) => {
                lhs.with_collapsed_value_mut(|lhs| lhs.clone() + rhs.clone())
            }
        }
        .map(ValueContainer::Local)
    }
}

impl Sub<ValueContainer> for ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    fn sub(self, rhs: ValueContainer) -> Self::Output {
        (&self).sub(&rhs)
    }
}

impl Sub<&ValueContainer> for &ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    fn sub(self, rhs: &ValueContainer) -> Self::Output {
        match (self, rhs) {
            (ValueContainer::Local(lhs), ValueContainer::Local(rhs)) => {
                lhs - rhs
            }
            (ValueContainer::Shared(lhs), ValueContainer::Shared(rhs)) => lhs
                .with_collapsed_value_mut(|lhs| {
                    rhs.with_collapsed_value_mut(|rhs| lhs.clone() - rhs.clone())
                }),
            (ValueContainer::Local(lhs), ValueContainer::Shared(rhs)) => {
                rhs.with_collapsed_value_mut(|rhs| lhs - rhs)
            }
            (ValueContainer::Shared(lhs), ValueContainer::Local(rhs)) => {
                lhs.with_collapsed_value_mut(|lhs| lhs.clone() - rhs.clone())
            }
        }
        .map(ValueContainer::Local)
    }
}

impl Neg for ValueContainer {
    type Output = Result<ValueContainer, ValueError>;

    fn neg(self) -> Self::Output {
        match self {
            ValueContainer::Local(value) => (-value).map(ValueContainer::Local),
            ValueContainer::Shared(reference) => reference
                .with_collapsed_value_mut(|value| {
                    (-value.clone()).map(ValueContainer::Local)
                }),
        }
    }
}
