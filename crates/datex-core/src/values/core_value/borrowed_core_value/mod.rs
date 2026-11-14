use crate::{
    prelude::*,
    types::{
        entities::entity_type_definition::EntityTypeDefinition, r#type::Type,
    },
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean,
            callable::Callable,
            decimal::{Decimal, typed_decimal::TypedDecimal},
            endpoint::Endpoint,
            instant::Instant,
            integer::{Integer, typed_integer::TypedInteger},
            list::List,
            map::Map,
            range::Range,
        },
        value::{
            borrowed_value::BorrowedValue,
            value_classification::ValueClassification,
        },
        value_container::ValueContainer,
    },
};
pub mod equality;
pub mod hash;
#[cfg(feature = "ast")]
mod to_datex_expression_data;

use crate::values::{
    core_value_with_classification::borrowed_core_value_with_classification::BorrowedCoreValueWithClassification,
    core_values::{
        receiver::ValueContainerReceiver, sender::ValueContainerSender,
    },
};
use core::ops::Deref;

/// Similar to [CoreValue], but it is a potentially borrowed reference to a [CoreValue] variant instead of owning it.
#[derive(Debug, Default)]
pub enum BorrowedCoreValue<'a> {
    #[default]
    Uninitialized,
    Null,
    Boolean(Goat<'a, Boolean>),
    Integer(Goat<'a, Integer>),
    TypedInteger(Goat<'a, TypedInteger>),
    Decimal(Goat<'a, Decimal>),
    TypedDecimal(Goat<'a, TypedDecimal>),
    Text(Goat<'a, str>),
    Endpoint(Goat<'a, Endpoint>),
    List(Goat<'a, List>),
    Map(Goat<'a, Map>),
    Type(Goat<'a, Type>),
    EntityTypeDefinition(Goat<'a, EntityTypeDefinition>),
    Callable(Goat<'a, Callable>),
    Range(Goat<'a, Range>),
    Box(Goat<'a, Box<ValueContainer>>),
    Instant(Goat<'a, Instant>),
    Sender(Goat<'a, ValueContainerSender>),
    Receiver(Goat<'a, ValueContainerReceiver>),
}

impl<'a> BorrowedCoreValue<'a> {
    pub(crate) fn clone_to_core_value(&self) -> CoreValue {
        match self {
            BorrowedCoreValue::Uninitialized => CoreValue::Uninitialized,
            BorrowedCoreValue::Null => CoreValue::Null,
            BorrowedCoreValue::Boolean(boolean) => {
                CoreValue::Boolean(boolean.deref().clone())
            }
            BorrowedCoreValue::Integer(integer) => {
                CoreValue::Integer(integer.deref().clone())
            }
            BorrowedCoreValue::TypedInteger(typed_integer) => {
                CoreValue::TypedInteger(typed_integer.deref().clone())
            }
            BorrowedCoreValue::Decimal(decimal) => {
                CoreValue::Decimal(decimal.deref().clone())
            }
            BorrowedCoreValue::TypedDecimal(typed_decimal) => {
                CoreValue::TypedDecimal(typed_decimal.deref().clone())
            }
            BorrowedCoreValue::Text(text) => {
                CoreValue::Text(text.to_string().into())
            }
            BorrowedCoreValue::Endpoint(endpoint) => {
                CoreValue::Endpoint(endpoint.deref().clone())
            }
            BorrowedCoreValue::List(list) => {
                CoreValue::List(list.deref().clone())
            }
            BorrowedCoreValue::Map(map) => CoreValue::Map(map.deref().clone()),
            BorrowedCoreValue::Type(type_value) => {
                CoreValue::Type(type_value.deref().clone())
            }
            BorrowedCoreValue::EntityTypeDefinition(entity_type_definition) => {
                CoreValue::EntityTypeDefinition(
                    entity_type_definition.deref().clone(),
                )
            }
            BorrowedCoreValue::Callable(callable) => {
                CoreValue::Callable(callable.deref().clone())
            }
            BorrowedCoreValue::Range(range) => {
                CoreValue::Range(range.deref().clone())
            }
            BorrowedCoreValue::Box(boxed_value) => {
                CoreValue::Box(boxed_value.deref().clone())
            }
            BorrowedCoreValue::Instant(instant) => {
                CoreValue::Instant(*instant.deref())
            }
            BorrowedCoreValue::Sender(sender) => {
                CoreValue::Sender(sender.deref().clone())
            }
            BorrowedCoreValue::Receiver(_receiver) => {
                todo!("cannot clone receiver")
            }
        }
    }
}

impl<'a> From<&'a CoreValue> for BorrowedCoreValue<'a> {
    fn from(core_value: &'a CoreValue) -> Self {
        match core_value {
            CoreValue::Callable(callable) => {
                BorrowedCoreValue::Callable(Goat::Borrowed(callable))
            }
            CoreValue::Uninitialized => BorrowedCoreValue::Uninitialized,
            CoreValue::Null => BorrowedCoreValue::Null,
            CoreValue::Boolean(boolean) => {
                BorrowedCoreValue::Boolean(Goat::Borrowed(boolean))
            }
            CoreValue::Integer(integer) => {
                BorrowedCoreValue::Integer(Goat::Borrowed(integer))
            }
            CoreValue::TypedInteger(typed_integer) => {
                BorrowedCoreValue::TypedInteger(Goat::Borrowed(typed_integer))
            }
            CoreValue::Decimal(decimal) => {
                BorrowedCoreValue::Decimal(Goat::Borrowed(decimal))
            }
            CoreValue::TypedDecimal(typed_decimal) => {
                BorrowedCoreValue::TypedDecimal(Goat::Borrowed(typed_decimal))
            }
            CoreValue::Text(text) => {
                BorrowedCoreValue::Text(Goat::Borrowed(text.as_str()))
            }
            CoreValue::Endpoint(endpoint) => {
                BorrowedCoreValue::Endpoint(Goat::Borrowed(endpoint))
            }
            CoreValue::List(list) => {
                BorrowedCoreValue::List(Goat::Borrowed(list))
            }
            CoreValue::Map(map) => BorrowedCoreValue::Map(Goat::Borrowed(map)),
            CoreValue::Type(type_value) => {
                BorrowedCoreValue::Type(Goat::Borrowed(type_value))
            }
            CoreValue::EntityTypeDefinition(entity_type_definition) => {
                BorrowedCoreValue::EntityTypeDefinition(Goat::Borrowed(
                    entity_type_definition,
                ))
            }
            CoreValue::Range(range) => {
                BorrowedCoreValue::Range(Goat::Borrowed(range))
            }
            CoreValue::Box(boxed_value) => {
                BorrowedCoreValue::Box(Goat::Borrowed(boxed_value))
            }
            CoreValue::Instant(instant) => {
                BorrowedCoreValue::Instant(Goat::Borrowed(instant))
            }
            CoreValue::Sender(sender) => {
                BorrowedCoreValue::Sender(Goat::Borrowed(sender))
            }
            CoreValue::Receiver(receiver) => {
                BorrowedCoreValue::Receiver(Goat::Borrowed(receiver))
            }
        }
    }
}

impl<'a> From<BorrowedCoreValue<'a>> for BorrowedValue<'a> {
    fn from(borrowed_core_value: BorrowedCoreValue<'a>) -> Self {
        BorrowedValue::Core(BorrowedCoreValueWithClassification {
            inner: borrowed_core_value,
            classification: ValueClassification::new_unclassified(),
        })
    }
}

#[derive(Default, Debug)]
pub enum BorrowedCoreValueMut<'a> {
    #[default]
    Uninitialized,
    Null,
    Boolean(GoatMut<'a, Boolean>),
    Integer(GoatMut<'a, Integer>),
    TypedInteger(GoatMut<'a, TypedInteger>),
    Decimal(GoatMut<'a, Decimal>),
    TypedDecimal(GoatMut<'a, TypedDecimal>),
    Text(GoatMut<'a, str>),
    Endpoint(GoatMut<'a, Endpoint>),
    List(GoatMut<'a, List>),
    Map(GoatMut<'a, Map>),
    Type(GoatMut<'a, Type>),
    EntityTypeDefinition(GoatMut<'a, EntityTypeDefinition>),
    Callable(GoatMut<'a, Callable>),
    Range(GoatMut<'a, Range>),
    Box(GoatMut<'a, Box<ValueContainer>>),
    Instant(GoatMut<'a, Instant>),
    Sender(GoatMut<'a, ValueContainerSender>),
    Receiver(GoatMut<'a, ValueContainerReceiver>),
}

impl<'a> BorrowedCoreValueMut<'a> {
    pub fn try_clone_to_core_value(&self) -> Option<CoreValue> {
        match self {
            BorrowedCoreValueMut::Uninitialized => {
                Some(CoreValue::Uninitialized)
            }
            BorrowedCoreValueMut::Null => Some(CoreValue::Null),
            BorrowedCoreValueMut::Boolean(boolean) => {
                Some(CoreValue::Boolean(boolean.deref().clone()))
            }
            BorrowedCoreValueMut::Integer(integer) => {
                Some(CoreValue::Integer(integer.deref().clone()))
            }
            BorrowedCoreValueMut::TypedInteger(typed_integer) => {
                Some(CoreValue::TypedInteger(typed_integer.deref().clone()))
            }
            BorrowedCoreValueMut::Decimal(decimal) => {
                Some(CoreValue::Decimal(decimal.deref().clone()))
            }
            BorrowedCoreValueMut::TypedDecimal(typed_decimal) => {
                Some(CoreValue::TypedDecimal(typed_decimal.deref().clone()))
            }
            BorrowedCoreValueMut::Text(text) => {
                Some(CoreValue::Text(text.to_string().into()))
            }
            BorrowedCoreValueMut::Endpoint(endpoint) => {
                Some(CoreValue::Endpoint(endpoint.deref().clone()))
            }
            BorrowedCoreValueMut::List(list) => {
                Some(CoreValue::List(list.deref().clone()))
            }
            BorrowedCoreValueMut::Map(map) => {
                Some(CoreValue::Map(map.deref().clone()))
            }
            BorrowedCoreValueMut::Type(type_value) => {
                Some(CoreValue::Type(type_value.deref().clone()))
            }
            BorrowedCoreValueMut::EntityTypeDefinition(
                entity_type_definition,
            ) => Some(CoreValue::EntityTypeDefinition(
                entity_type_definition.deref().clone(),
            )),
            BorrowedCoreValueMut::Callable(callable) => {
                Some(CoreValue::Callable(callable.deref().clone()))
            }
            BorrowedCoreValueMut::Range(range) => {
                Some(CoreValue::Range(range.deref().clone()))
            }
            BorrowedCoreValueMut::Box(boxed_value) => {
                Some(CoreValue::Box(boxed_value.deref().clone()))
            }
            BorrowedCoreValueMut::Instant(instant) => {
                Some(CoreValue::Instant(*instant.deref()))
            }
            BorrowedCoreValueMut::Sender(sender) => {
                Some(CoreValue::Sender(sender.deref().clone()))
            }
            BorrowedCoreValueMut::Receiver(_receiver) => {
                todo!("cannot clone receiver")
            }
        }
    }
}

impl<'a> TryFrom<&'a mut CoreValue> for BorrowedCoreValueMut<'a> {
    type Error = ();

    /// Attempts to create a [BorrowedCoreValueMut] from a mutable reference to a [CoreValue].
    /// Returns an error if the conversion is not possible, like mutable borrowing a static string slice.
    fn try_from(core_value: &'a mut CoreValue) -> Result<Self, Self::Error> {
        Ok(match core_value {
            CoreValue::Callable(callable) => {
                BorrowedCoreValueMut::Callable(GoatMut::Borrowed(callable))
            }
            CoreValue::Uninitialized => BorrowedCoreValueMut::Uninitialized,
            CoreValue::Null => BorrowedCoreValueMut::Null,
            CoreValue::Boolean(boolean) => {
                BorrowedCoreValueMut::Boolean(GoatMut::Borrowed(boolean))
            }
            CoreValue::Integer(integer) => {
                BorrowedCoreValueMut::Integer(GoatMut::Borrowed(integer))
            }
            CoreValue::TypedInteger(typed_integer) => {
                BorrowedCoreValueMut::TypedInteger(GoatMut::Borrowed(
                    typed_integer,
                ))
            }
            CoreValue::Decimal(decimal) => {
                BorrowedCoreValueMut::Decimal(GoatMut::Borrowed(decimal))
            }
            CoreValue::TypedDecimal(typed_decimal) => {
                BorrowedCoreValueMut::TypedDecimal(GoatMut::Borrowed(
                    typed_decimal,
                ))
            }
            CoreValue::Text(text) => BorrowedCoreValueMut::Text(
                GoatMut::Borrowed(text.try_as_mut_str().ok_or(())?),
            ),
            CoreValue::Endpoint(endpoint) => {
                BorrowedCoreValueMut::Endpoint(GoatMut::Borrowed(endpoint))
            }
            CoreValue::List(list) => {
                BorrowedCoreValueMut::List(GoatMut::Borrowed(list))
            }
            CoreValue::Map(map) => {
                BorrowedCoreValueMut::Map(GoatMut::Borrowed(map))
            }
            CoreValue::Type(type_value) => {
                BorrowedCoreValueMut::Type(GoatMut::Borrowed(type_value))
            }
            CoreValue::EntityTypeDefinition(entity_type_definition) => {
                BorrowedCoreValueMut::EntityTypeDefinition(GoatMut::Borrowed(
                    entity_type_definition,
                ))
            }
            CoreValue::Range(range) => {
                BorrowedCoreValueMut::Range(GoatMut::Borrowed(range))
            }
            CoreValue::Box(boxed_value) => {
                BorrowedCoreValueMut::Box(GoatMut::Borrowed(boxed_value))
            }
            CoreValue::Instant(instant) => {
                BorrowedCoreValueMut::Instant(GoatMut::Borrowed(instant))
            }
            CoreValue::Sender(sender) => {
                BorrowedCoreValueMut::Sender(GoatMut::Borrowed(sender))
            }
            CoreValue::Receiver(receiver) => {
                BorrowedCoreValueMut::Receiver(GoatMut::Borrowed(receiver))
            }
        })
    }
}

impl<'a> From<&'a mut str> for BorrowedCoreValueMut<'a> {
    fn from(s: &'a mut str) -> Self {
        BorrowedCoreValueMut::Text(GoatMut::Borrowed(s))
    }
}

impl<'a> From<&'a str> for BorrowedCoreValue<'a> {
    fn from(s: &'a str) -> Self {
        BorrowedCoreValue::Text(Goat::Borrowed(s))
    }
}
