use core::fmt::{Debug, Pointer};
use core::ops::{Deref, DerefMut};
use log::info;
use crate::traits::try_clone::TryClone;
use crate::types::entities::entity_type_definition::EntityTypeDefinition;
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;
use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::prelude::*;
use crate::values::core_value::CoreValue;
use crate::values::core_values::boolean::Boolean;
use crate::values::core_values::callable::Callable;
use crate::values::core_values::decimal::Decimal;
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;
use crate::values::core_values::endpoint::Endpoint;
use crate::values::core_values::integer::Integer;
use crate::values::core_values::integer::typed_integer::TypedInteger;
use crate::values::core_values::list::List;
use crate::values::core_values::map::Map;
use crate::values::core_values::native::{DatexNative, NativeCoreValue};
use crate::values::core_values::range::Range;
use crate::values::core_values::text::Text;
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

/// Similar to [Value], but contains a [BorrowedCoreValue] instead of a [CoreValue].
/// It is used to represent a potentially borrowed reference to a [CoreValue] variant instead of owning it.
#[derive(Debug)]
pub struct BorrowedValue<'a> {
    pub(crate) inner: BorrowedCoreValue<'a>,
    pub(crate) custom_type: Option<TypeDefinition>,
}

impl<'a> BorrowedValue<'a> {
    pub fn try_clone_to_value(self) -> Result<Value, ()>
    where
        CoreValue: Clone,
    {
       Ok(Value {
           inner: self.inner.try_clone_to_core_value()?,
           custom_type: self.custom_type,
       })
    }
}


impl<'a> From<&'a Value> for BorrowedValue<'a> {
    fn from(value: &'a Value) -> Self {
        BorrowedValue {
            inner: BorrowedCoreValue::from(&value.inner),
            custom_type: value.custom_type.clone(),
        }
    }
}

/// Similar to [CoreValue], but it is a potentially borrowed reference to a [CoreValue] variant instead of owning it.
#[derive(Default)]
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

    pub fn try_clone_to_core_value(self) -> Result<CoreValue, ()>
    {
        match self {
            BorrowedCoreValue::Uninitialized => Ok(CoreValue::Uninitialized),
            BorrowedCoreValue::Null => Ok(CoreValue::Null),
            BorrowedCoreValue::Boolean(boolean) => Ok(CoreValue::Boolean(boolean.deref().clone())),
            BorrowedCoreValue::Integer(integer) => Ok(CoreValue::Integer(integer.deref().clone())),
            BorrowedCoreValue::TypedInteger(typed_integer) => Ok(CoreValue::TypedInteger(typed_integer.deref().clone())),
            BorrowedCoreValue::Decimal(decimal) => Ok(CoreValue::Decimal(decimal.deref().clone())),
            BorrowedCoreValue::TypedDecimal(typed_decimal) => Ok(CoreValue::TypedDecimal(typed_decimal.deref().clone())),
            BorrowedCoreValue::Text(text) => Ok(CoreValue::Text(text.deref().clone())),
            BorrowedCoreValue::Endpoint(endpoint) => Ok(CoreValue::Endpoint(endpoint.deref().clone())),
            BorrowedCoreValue::List(list) => Ok(CoreValue::List(list.deref().clone())),
            BorrowedCoreValue::Map(map) => Ok(CoreValue::Map(map.deref().clone())),
            BorrowedCoreValue::Type(type_value) => Ok(CoreValue::Type(type_value.deref().clone())),
            BorrowedCoreValue::EntityTypeDefinition(entity_type_definition) => Ok(CoreValue::EntityTypeDefinition(entity_type_definition.deref().clone())),
            BorrowedCoreValue::Callable(callable) => Ok(CoreValue::Callable(callable.deref().clone())),
            BorrowedCoreValue::Range(range) => Ok(CoreValue::Range(range.deref().clone())),
            BorrowedCoreValue::Box(boxed_value) => Ok(CoreValue::Box(boxed_value.deref().clone())),
            BorrowedCoreValue::Native(native) => native.deref().try_clone(),
        }
    }
}

impl<'a> From<&'a CoreValue> for BorrowedCoreValue<'a> {
    fn from(core_value: &'a CoreValue) -> Self {
        match core_value {
            CoreValue::Callable(callable) => BorrowedCoreValue::Callable(Goat::Borrowed(callable)),
            CoreValue::Native(native) => BorrowedCoreValue::Native(Goat::Borrowed(native.value.deref())),
            CoreValue::Uninitialized => BorrowedCoreValue::Uninitialized,
            CoreValue::Null => BorrowedCoreValue::Null,
            CoreValue::Boolean(boolean) => BorrowedCoreValue::Boolean(Goat::Borrowed(boolean)),
            CoreValue::Integer(integer) => BorrowedCoreValue::Integer(Goat::Borrowed(integer)),
            CoreValue::TypedInteger(typed_integer) => BorrowedCoreValue::TypedInteger(Goat::Borrowed(typed_integer)),
            CoreValue::Decimal(decimal) => BorrowedCoreValue::Decimal(Goat::Borrowed(decimal)),
            CoreValue::TypedDecimal(typed_decimal) => BorrowedCoreValue::TypedDecimal(Goat::Borrowed(typed_decimal)),
            CoreValue::Text(text) => BorrowedCoreValue::Text(Goat::Borrowed(text)),
            CoreValue::Endpoint(endpoint) => BorrowedCoreValue::Endpoint(Goat::Borrowed(endpoint)),
            CoreValue::List(list) => BorrowedCoreValue::List(Goat::Borrowed(list)),
            CoreValue::Map(map) => BorrowedCoreValue::Map(Goat::Borrowed(map)),
            CoreValue::Type(type_value) => BorrowedCoreValue::Type(Goat::Borrowed(type_value)),
            CoreValue::EntityTypeDefinition(entity_type_definition) => BorrowedCoreValue::EntityTypeDefinition(Goat::Borrowed(entity_type_definition)),
            CoreValue::Range(range) => BorrowedCoreValue::Range(Goat::Borrowed(range)),
            CoreValue::Box(boxed_value) => BorrowedCoreValue::Box(Goat::Borrowed(boxed_value)),
        }
    }
}

impl Debug for BorrowedCoreValue<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BorrowedCoreValue::Uninitialized => write!(f, "Uninitialized"),
            BorrowedCoreValue::Null => write!(f, "Null"),
            BorrowedCoreValue::Boolean(boolean) => boolean.fmt(f),
            BorrowedCoreValue::Integer(integer) => integer.fmt(f),
            BorrowedCoreValue::TypedInteger(typed_integer) => typed_integer.fmt(f),
            BorrowedCoreValue::Decimal(decimal) => decimal.fmt(f),
            BorrowedCoreValue::TypedDecimal(typed_decimal) => typed_decimal.fmt(f),
            BorrowedCoreValue::Text(text) => text.fmt(f),
            BorrowedCoreValue::Endpoint(endpoint) => endpoint.fmt(f),
            BorrowedCoreValue::List(list) => list.fmt(f),
            BorrowedCoreValue::Map(map) => map.fmt(f),
            BorrowedCoreValue::Type(type_value) => type_value.fmt(f),
            BorrowedCoreValue::EntityTypeDefinition(entity_type_definition) => entity_type_definition.fmt(f),
            BorrowedCoreValue::Callable(callable) => callable.fmt(f),
            BorrowedCoreValue::Range(range) => range.fmt(f),
            BorrowedCoreValue::Box(boxed_value) => boxed_value.fmt(f),
            BorrowedCoreValue::Native(native) => native.fmt(f),
        }
    }
}


/// Similar to [Value], but contains a [BorrowedCoreValueMut] instead of a [CoreValue].
/// It is used to represent a potentially borrowed mutable reference to a [CoreValue] variant instead of owning it.
pub struct BorrowedValueMut<'a> {
    pub(crate) inner: BorrowedCoreValueMut<'a>,
    pub(crate) custom_type: Option<TypeDefinition>,
}
impl<'a> From<&'a mut Value> for BorrowedValueMut<'a> {
    fn from(value: &'a mut Value) -> Self {
        BorrowedValueMut {
            inner: BorrowedCoreValueMut::from(&mut value.inner),
            custom_type: value.custom_type.clone(),
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
            CoreValue::Callable(callable) => BorrowedCoreValueMut::Callable(GoatMut::Borrowed(callable)),
            CoreValue::Native(native) => BorrowedCoreValueMut::Native(GoatMut::Borrowed(native.value.deref_mut())),
            CoreValue::Uninitialized => BorrowedCoreValueMut::Uninitialized,
            CoreValue::Null => BorrowedCoreValueMut::Null,
            CoreValue::Boolean(boolean) => BorrowedCoreValueMut::Boolean(GoatMut::Borrowed(boolean)),
            CoreValue::Integer(integer) => BorrowedCoreValueMut::Integer(GoatMut::Borrowed(integer)),
            CoreValue::TypedInteger(typed_integer) => BorrowedCoreValueMut::TypedInteger(GoatMut::Borrowed(typed_integer)),
            CoreValue::Decimal(decimal) => BorrowedCoreValueMut::Decimal(GoatMut::Borrowed(decimal)),
            CoreValue::TypedDecimal(typed_decimal) => BorrowedCoreValueMut::TypedDecimal(GoatMut::Borrowed(typed_decimal)),
            CoreValue::Text(text) => BorrowedCoreValueMut::Text(GoatMut::Borrowed(text)),
            CoreValue::Endpoint(endpoint) => BorrowedCoreValueMut::Endpoint(GoatMut::Borrowed(endpoint)),
            CoreValue::List(list) => BorrowedCoreValueMut::List(GoatMut::Borrowed(list)),
            CoreValue::Map(map) => BorrowedCoreValueMut::Map(GoatMut::Borrowed(map)),
            CoreValue::Type(type_value) => BorrowedCoreValueMut::Type(GoatMut::Borrowed(type_value)),
            CoreValue::EntityTypeDefinition(entity_type_definition) => BorrowedCoreValueMut::EntityTypeDefinition(GoatMut::Borrowed(entity_type_definition)),
            CoreValue::Range(range) => BorrowedCoreValueMut::Range(GoatMut::Borrowed(range)),
            CoreValue::Box(boxed_value) => BorrowedCoreValueMut::Box(GoatMut::Borrowed(boxed_value)),
        }
    }
}