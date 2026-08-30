use crate::{
    prelude::*,
    traits::try_clone::TryClone,
    types::{
        entities::entity_type_definition::EntityTypeDefinition, r#type::Type,
        type_definition::TypeDefinition,
    },
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean,
            callable::Callable,
            decimal::{Decimal, typed_decimal::TypedDecimal},
            endpoint::Endpoint,
            integer::{Integer, typed_integer::TypedInteger},
            list::List,
            map::Map,
            native::DatexNative,
            range::Range,
            text::Text,
        },
        value::Value,
        value_container::ValueContainer,
    },
};
use core::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use core::cell::{Ref, RefMut};
use crate::network::com_interfaces::default_setup_data::http_common::TLSMode;
use crate::preludes::derive::{BorrowedValueContainer, DatexNativeOnlyStructural, SharedReferencesCache};
use crate::shared_values::PointerAddress;
use crate::values::value::ValueClassification;

/// Similar to [Value], but contains a [BorrowedCoreValue] instead of a [CoreValue].
/// It is used to represent a potentially borrowed reference to a [CoreValue] variant instead of owning it.
#[derive(Debug)]
pub struct BorrowedValue<'a> {
    pub inner: BorrowedCoreValue<'a>,
    pub classification: ValueClassification,
}

/// Converts a [Goat] of a native value into a [Goat] of a dynamic [DatexNative] trait object.
pub fn into_dyn_goat<'a, T: DatexNative>(val: Goat<'a, T>) -> Goat<'a, dyn DatexNative> {
    match val {
        Goat::Ref(value) => {
            Goat::Ref(Ref::map(value, |value| value as &dyn DatexNative))
        }
        Goat::Borrowed(value) => {
            Goat::Borrowed(value)
        }
    }
}

/// Converts a [GoatMut] of a native value into a [GoatMut] of a dynamic [DatexNative] trait object.
pub fn into_dyn_goat_mut<'a, T: DatexNative>(val: GoatMut<'a, T>) -> GoatMut<'a, dyn DatexNative> {
    match val {
        GoatMut::Ref(value) => {
            GoatMut::Ref(RefMut::map(value, |value| value as &mut dyn DatexNative))
        }
        GoatMut::Borrowed(value) => {
            GoatMut::Borrowed(value)
        }
    }
}

impl<'a> BorrowedValue<'a> {
    /// Creates a new [BorrowedValue] from a reference to a native value.
    pub fn native_borrowed<T: DatexNative>(
        val: impl Into<Goat<'a, T>>,
        cache: &mut SharedReferencesCache,
    ) -> Self {
        let val = into_dyn_goat(val.into());
        let entity_type = val.entity_type(cache);
        BorrowedValue {
            inner: BorrowedCoreValue::Native(val),
            classification: ValueClassification::from(entity_type),
        }
    }

    /// Creates a new [BorrowedValue] from a reference to a native value.
    pub fn native_borrowed_only_structural<T: DatexNativeOnlyStructural>(
        val: impl Into<Goat<'a, T>>
    ) -> Self {
        let val = into_dyn_goat(val.into());
        BorrowedValue {
            inner: BorrowedCoreValue::Native(val),
            classification: ValueClassification::None,
        }
    }

    pub fn try_clone_to_value(self) -> Result<Value, ()>
    where
        CoreValue: Clone,
    {
        Ok(Value {
            inner: self.inner.try_clone_to_core_value()?,
            classification: self.classification.clone(),
        })
    }
}

impl<'a> From<&'a Value> for BorrowedValue<'a> {
    fn from(value: &'a Value) -> Self {
        BorrowedValue {
            inner: BorrowedCoreValue::from(&value.inner),
            classification: value.classification.clone(),
        }
    }
}

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
    Text(Goat<'a, Text>),
    Endpoint(Goat<'a, Endpoint>),
    List(Goat<'a, List>),
    Map(Goat<'a, Map>),
    Type(Goat<'a, Type>),
    EntityTypeDefinition(Goat<'a, EntityTypeDefinition>),
    Callable(Goat<'a, Callable>),
    Range(Goat<'a, Range>),
    Box(Goat<'a, Box<ValueContainer>>),
    Native(Goat<'a, dyn DatexNative>),
}

impl<'a> BorrowedCoreValue<'a> {
    /// Tries to get a borrow of the current value as the specified type.
    /// Does not perform any type conversion.
    pub fn try_as<T>(self) -> Option<Goat<'a, T>>
    where
        Goat<'a, T>: TryFrom<BorrowedCoreValue<'a>>,
    {
        Goat::try_from(self).ok()
    }

    pub fn try_clone_to_core_value(self) -> Result<CoreValue, ()> {
        match self {
            BorrowedCoreValue::Uninitialized => Ok(CoreValue::Uninitialized),
            BorrowedCoreValue::Null => Ok(CoreValue::Null),
            BorrowedCoreValue::Boolean(boolean) => {
                Ok(CoreValue::Boolean(boolean.deref().clone()))
            }
            BorrowedCoreValue::Integer(integer) => {
                Ok(CoreValue::Integer(integer.deref().clone()))
            }
            BorrowedCoreValue::TypedInteger(typed_integer) => {
                Ok(CoreValue::TypedInteger(typed_integer.deref().clone()))
            }
            BorrowedCoreValue::Decimal(decimal) => {
                Ok(CoreValue::Decimal(decimal.deref().clone()))
            }
            BorrowedCoreValue::TypedDecimal(typed_decimal) => {
                Ok(CoreValue::TypedDecimal(typed_decimal.deref().clone()))
            }
            BorrowedCoreValue::Text(text) => {
                Ok(CoreValue::Text(text.deref().clone()))
            }
            BorrowedCoreValue::Endpoint(endpoint) => {
                Ok(CoreValue::Endpoint(endpoint.deref().clone()))
            }
            BorrowedCoreValue::List(list) => {
                Ok(CoreValue::List(list.deref().clone()))
            }
            BorrowedCoreValue::Map(map) => {
                Ok(CoreValue::Map(map.deref().clone()))
            }
            BorrowedCoreValue::Type(type_value) => {
                Ok(CoreValue::Type(type_value.deref().clone()))
            }
            BorrowedCoreValue::EntityTypeDefinition(entity_type_definition) => {
                Ok(CoreValue::EntityTypeDefinition(
                    entity_type_definition.deref().clone(),
                ))
            }
            BorrowedCoreValue::Callable(callable) => {
                Ok(CoreValue::Callable(callable.deref().clone()))
            }
            BorrowedCoreValue::Range(range) => {
                Ok(CoreValue::Range(range.deref().clone()))
            }
            BorrowedCoreValue::Box(boxed_value) => {
                Ok(CoreValue::Box(boxed_value.deref().clone()))
            }
            BorrowedCoreValue::Native(native) => native.deref().try_clone(),
        }
    }
}

impl<'a> From<&'a CoreValue> for BorrowedCoreValue<'a> {
    fn from(core_value: &'a CoreValue) -> Self {
        match core_value {
            CoreValue::Callable(callable) => {
                BorrowedCoreValue::Callable(Goat::Borrowed(callable))
            }
            CoreValue::Native(native) => {
                BorrowedCoreValue::Native(Goat::Borrowed(native.value.deref()))
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
                BorrowedCoreValue::Text(Goat::Borrowed(text))
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
        }
    }
}

impl<'a> From<BorrowedCoreValue<'a>> for BorrowedValue<'a> {
    fn from(borrowed_core_value: BorrowedCoreValue<'a>) -> Self {
        BorrowedValue {
            inner: borrowed_core_value,
            classification: ValueClassification::None,
        }
    }
}

/// Similar to [Value], but contains a [BorrowedCoreValueMut] instead of a [CoreValue].
/// It is used to represent a potentially borrowed mutable reference to a [CoreValue] variant instead of owning it.
pub struct BorrowedValueMut<'a> {
    pub(crate) inner: BorrowedCoreValueMut<'a>,
    pub(crate) classification: ValueClassification,
}

impl<'a> BorrowedValueMut<'a> {
    /// Creates a new [BorrowedValueMut] from a reference to a native value.
    pub fn native_borrowed<T: DatexNative>(
        val: impl Into<GoatMut<'a, T>>,
        cache: &mut SharedReferencesCache,
    ) -> Self {
        let val = into_dyn_goat_mut(val.into());
        let entity_type = val.entity_type(cache);
        BorrowedValueMut {
            inner: BorrowedCoreValueMut::Native(val),
            classification: ValueClassification::from(entity_type),
        }
    }

    /// Creates a new [BorrowedValueMut] from a reference to a native value.
    pub fn native_borrowed_only_structural<T: DatexNativeOnlyStructural>(
        val: impl Into<GoatMut<'a, T>>
    ) -> Self {
        let val = into_dyn_goat_mut(val.into());
        BorrowedValueMut {
            inner: BorrowedCoreValueMut::Native(val),
            classification: ValueClassification::None,
        }
    }
}

impl<'a> From<&'a mut Value> for BorrowedValueMut<'a> {
    fn from(value: &'a mut Value) -> Self {
        BorrowedValueMut {
            inner: BorrowedCoreValueMut::from(&mut value.inner),
            classification: value.classification.clone(),
        }
    }
}

#[derive(Default)]
pub enum BorrowedCoreValueMut<'a> {
    #[default]
    Uninitialized,
    Null,
    Boolean(GoatMut<'a, Boolean>),
    Integer(GoatMut<'a, Integer>),
    TypedInteger(GoatMut<'a, TypedInteger>),
    Decimal(GoatMut<'a, Decimal>),
    TypedDecimal(GoatMut<'a, TypedDecimal>),
    Text(GoatMut<'a, Text>),
    Endpoint(GoatMut<'a, Endpoint>),
    List(GoatMut<'a, List>),
    Map(GoatMut<'a, Map>),
    Type(GoatMut<'a, Type>),
    EntityTypeDefinition(GoatMut<'a, EntityTypeDefinition>),
    Callable(GoatMut<'a, Callable>),
    Range(GoatMut<'a, Range>),
    Box(GoatMut<'a, Box<ValueContainer>>),
    Native(GoatMut<'a, dyn DatexNative>),
}

impl<'a> BorrowedCoreValueMut<'a> {
    /// Tries to get a borrow of the current value as the specified type.
    /// Does not perform any type conversion.
    pub fn try_as<T>(self) -> Option<Goat<'a, T>>
    where
        Goat<'a, T>: TryFrom<BorrowedCoreValueMut<'a>>,
    {
        Goat::try_from(self).ok()
    }

    /// Tries to get a mutable borrow of the current value as the specified type.
    /// Does not perform any type conversion.
    pub fn try_as_mut<T>(self) -> Option<GoatMut<'a, T>>
    where
        GoatMut<'a, T>: TryFrom<BorrowedCoreValueMut<'a>>,
    {
        GoatMut::try_from(self).ok()
    }
}

impl<'a> From<&'a mut CoreValue> for BorrowedCoreValueMut<'a> {
    fn from(core_value: &'a mut CoreValue) -> Self {
        match core_value {
            CoreValue::Callable(callable) => {
                BorrowedCoreValueMut::Callable(GoatMut::Borrowed(callable))
            }
            CoreValue::Native(native) => BorrowedCoreValueMut::Native(
                GoatMut::Borrowed(native.value.deref_mut()),
            ),
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
            CoreValue::Text(text) => {
                BorrowedCoreValueMut::Text(GoatMut::Borrowed(text))
            }
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
        }
    }
}
