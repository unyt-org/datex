use crate::{
    prelude::*,
    runtime::execution::ExecutionError,
    traits::{apply::Apply, structural_eq::StructuralEq, value_eq::ValueEq},
    types::type_definition::TypeDefinition,
    values::{
        core_value::CoreValue,
        core_values::{
            callable::{Callable, CallableBody, CallableSignature},
            integer::typed_integer::TypedInteger,
        },
        value_container::{
            ValueContainer, error::ValueError, value_key::BorrowedValueKey,
        },
    },
};
pub mod apply;
pub mod serde_dif;
use crate::{
    runtime::memory::Memory,
    shared_values::{errors::AccessError, observers::TransceiverId},
    types::r#type::Type,
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData,
        },
        update_handler::UpdateHandler,
    },
};
use core::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Deref, Neg, Not, Sub},
    result::Result,
};
use log::error;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Value {
    pub inner: CoreValue,
    // actual type of the value - if [None], use default type for given value
    pub custom_type: Option<Type>,
}

/// Two values are structurally equal, if their inner values are structurally equal, regardless
/// of the actual_type of the values
impl StructuralEq for Value {
    fn structural_eq(&self, other: &Self) -> bool {
        self.inner.structural_eq(&other.inner)
    }
}

/// Value equality corresponds to partial equality:
/// Both type and inner value are the same
impl ValueEq for Value {
    fn value_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Deref for Value {
    type Target = CoreValue;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Into<CoreValue>> From<T> for Value {
    fn from(inner: T) -> Self {
        let inner = inner.into();
        Value {
            inner,
            custom_type: None,
        }
    }
}
impl Value {
    pub fn null() -> Self {
        CoreValue::Null.into()
    }
}

impl Value {
    pub fn callable(
        name: Option<String>,
        signature: CallableSignature,
        body: CallableBody,
    ) -> Self {
        Value {
            inner: CoreValue::Callable(Callable {
                name,
                signature: signature.clone(),
                body,
            }),
            custom_type: Some(Type::from(TypeDefinition::callable(signature))),
        }
    }

    pub fn is_type(&self) -> bool {
        core::matches!(self.inner, CoreValue::Type(_))
    }
    pub fn is_null(&self) -> bool {
        core::matches!(self.inner, CoreValue::Null)
    }
    pub fn is_text(&self) -> bool {
        core::matches!(self.inner, CoreValue::Text(_))
    }
    pub fn is_integer_i8(&self) -> bool {
        core::matches!(
            &self.inner,
            CoreValue::TypedInteger(TypedInteger::I8(_))
        )
    }
    pub fn is_bool(&self) -> bool {
        core::matches!(self.inner, CoreValue::Boolean(_))
    }
    pub fn is_map(&self) -> bool {
        core::matches!(self.inner, CoreValue::Map(_))
    }
    pub fn is_list(&self) -> bool {
        core::matches!(self.inner, CoreValue::List(_))
    }

    /// Returns true if the current Value's actual type is the same as its default type
    /// E.g. if the type is integer for an Integer value, or integer/u8 for a typed integer value
    /// This will return false for an integer value if the actual type is one of the following:
    /// * an ImplType<integer, x>
    /// * a new nominal type containing an integer
    /// TODO #604: this does not match all cases of default types from the point of view of the compiler -
    /// integer variants (despite bigint) can be distinguished based on the instruction code, but for text variants,
    /// the variant must be included in the compiler output - so we need to handle theses cases as well.
    /// Generally speaking, all variants except the few integer variants should never be considered default types.
    pub fn has_default_type(&self, memory: &Memory) -> bool {
        match &self.custom_type {
            None => true,
            Some(Type::Nominal(nominal_type)) => {
                nominal_type == &self.default_nominal_type(memory)
            }
            Some(_) => false,
        }
    }

    /// Returns the actual type, generating the default type from the provided memory if no custom typoe is set
    pub fn actual_type(&self, memory: &Memory) -> Type {
        match &self.custom_type {
            Some(actual_type) => actual_type.clone(),
            None => Type::Nominal(self.default_nominal_type(memory)),
        }
    }

    /// Gets a property on the value if applicable (e.g. for map and structs)
    pub fn try_get_property<'a>(
        &self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, AccessError> {
        match self.inner {
            CoreValue::Map(ref map) => {
                // If the value is a map, get the property
                Ok(map.get(key)?.clone())
            }
            CoreValue::List(ref list) => {
                if let Some(index) = key.into().try_as_index() {
                    Ok(list.try_get(index)?.clone())
                } else {
                    Err(AccessError::InvalidIndexKey)
                }
            }
            CoreValue::Text(ref text) => {
                if let Some(index) = key.into().try_as_index() {
                    let char = text.char_at(index)?;
                    Ok(ValueContainer::from(char.to_string()))
                } else {
                    Err(AccessError::InvalidIndexKey)
                }
            }
            _ => {
                // If the value is not an map, we cannot get a property
                Err(AccessError::InvalidOperation(
                    "Cannot get property".to_string(),
                ))
            }
        }
    }

    /// Takes (removes) a property from the value if applicable (e.g. for map and structs)
    pub fn try_take_property<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<ValueContainer, AccessError> {
        match self.inner {
            CoreValue::Map(ref mut map) => {
                // If the value is a map, get the property
                Ok(map.try_delete(key)?)
            }
            CoreValue::List(ref mut list) => {
                if let Some(index) = key.into().try_as_index() {
                    Ok(list.delete(index)?)
                } else {
                    Err(AccessError::InvalidIndexKey)
                }
            }
            CoreValue::Text(ref text) => {
                if let Some(index) = key.into().try_as_index() {
                    let char = text.char_at(index)?;
                    Ok(ValueContainer::from(char.to_string()))
                } else {
                    Err(AccessError::InvalidIndexKey)
                }
            }
            _ => {
                // If the value is not an map, we cannot get a property
                Err(AccessError::InvalidOperation(
                    "Cannot get property".to_string(),
                ))
            }
        }
    }

    pub fn try_delete_property<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
    ) -> Result<(), AccessError> {
        match self.inner {
            CoreValue::Map(ref mut map) => {
                // If the value is a map, delete the property
                map.try_delete(key)?;
                Ok(())
            }
            CoreValue::List(ref mut list) => {
                if let Some(index) = key.into().try_as_index() {
                    list.delete(index)?;
                    Ok(())
                } else {
                    Err(AccessError::InvalidIndexKey)
                }
            }
            CoreValue::Text(_) => Err(AccessError::InvalidOperation(
                "Cannot delete property on text".to_string(),
            )),
            _ => {
                // If the value is not a map, we cannot delete a property
                Err(AccessError::InvalidOperation(
                    "Cannot delete property".to_string(),
                ))
            }
        }
    }

    /// Sets a property on the value if applicable (e.g. for maps)
    pub fn try_set_property<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
        val: ValueContainer,
    ) -> Result<(), AccessError> {
        let key = key.into();

        match self.inner {
            CoreValue::Map(ref mut map) => {
                // If the value is an map, set the property
                map.try_set(key, val)?;
            }
            CoreValue::List(ref mut list) => {
                if let Some(index) = key.try_as_index() {
                    list.try_set(index, val)
                        .map_err(AccessError::IndexOutOfBounds)?;
                } else {
                    return Err(AccessError::InvalidIndexKey);
                }
            }
            CoreValue::Text(ref mut text) => {
                if let Some(index) = key.try_as_index() {
                    if let ValueContainer::Local(v) = &val
                        && let CoreValue::Text(new_char) = &v.inner
                        && new_char.0.len() == 1
                    {
                        let char = new_char.0.chars().next().unwrap_or('\0');
                        text.set_char_at(index, char).map_err(|err| {
                            AccessError::IndexOutOfBounds(err)
                        })?;
                    } else {
                        return Err(AccessError::InvalidOperation(
                            "Can only set char character in text".to_string(),
                        ));
                    }
                } else {
                    return Err(AccessError::InvalidIndexKey);
                }
            }
            _ => {
                // If the value is not a map, we cannot set a property
                return Err(AccessError::InvalidOperation(format!(
                    "Cannot set property '{}' on non-map value: {:?}",
                    key, self
                )));
            }
        }

        Ok(())
    }
}

impl UpdateHandler for Value {
    fn try_replace(
        &mut self,
        data: ReplaceUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        match self.inner {
            CoreValue::Map(ref mut map) => map.try_replace(data, source_id),
            _ => Err(UpdateError::InvalidUpdate),
        }
    }

    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        match self.inner {
            CoreValue::Map(ref mut map) => map.try_set_entry(data, source_id),
            CoreValue::List(ref mut list) => {
                list.try_set_entry(data, source_id)
            }
            _ => Err(UpdateError::InvalidUpdate),
        }
    }

    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        match self.inner {
            CoreValue::Map(ref mut map) => {
                map.try_delete_entry(data, source_id)
            }
            CoreValue::List(ref mut list) => {
                list.try_delete_entry(data, source_id)
            }
            _ => Err(UpdateError::InvalidUpdate),
        }
    }

    fn try_append_entry(
        &mut self,
        data: AppendEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        match self.inner {
            CoreValue::Map(ref mut map) => {
                map.try_append_entry(data, source_id)
            }
            CoreValue::List(ref mut list) => {
                list.try_append_entry(data, source_id)
            }
            _ => Err(UpdateError::InvalidUpdate),
        }
    }

    fn try_clear(
        &mut self,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        match self.inner {
            CoreValue::Map(ref mut map) => map.try_clear(source_id),
            _ => Err(UpdateError::InvalidUpdate),
        }
    }

    fn try_list_splice(
        &mut self,
        _data: ListSpliceUpdateData,
        _source_id: TransceiverId,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        match self.inner {
            CoreValue::List(ref mut list) => {
                list.try_list_splice(_data, _source_id)
            }
            _ => Err(UpdateError::InvalidUpdate),
        }
    }
}

impl Add for Value {
    type Output = Result<Value, ValueError>;
    fn add(self, rhs: Value) -> Self::Output {
        Ok((&self.inner + &rhs.inner)?.into())
    }
}

impl Add for &Value {
    type Output = Result<Value, ValueError>;
    fn add(self, rhs: &Value) -> Self::Output {
        Value::add(self.clone(), rhs.clone())
    }
}

impl Sub for Value {
    type Output = Result<Value, ValueError>;
    fn sub(self, rhs: Value) -> Self::Output {
        Ok((&self.inner - &rhs.inner)?.into())
    }
}

impl Sub for &Value {
    type Output = Result<Value, ValueError>;
    fn sub(self, rhs: &Value) -> Self::Output {
        Value::sub(self.clone(), rhs.clone())
    }
}

impl Neg for Value {
    type Output = Result<Value, ValueError>;

    fn neg(self) -> Self::Output {
        (-self.inner).map(Value::from)
    }
}

impl Not for Value {
    type Output = Option<Value>;

    fn not(self) -> Self::Output {
        (!self.inner).map(Value::from)
    }
}

// TODO #119: crate a TryAddAssign trait etc.
impl<T> AddAssign<T> for Value
where
    Value: From<T>,
{
    fn add_assign(&mut self, rhs: T) {
        let rhs: Value = rhs.into();
        let res = self.inner.clone() + rhs.inner;
        if let Ok(res) = res {
            self.inner = res;
        } else {
            error!("Failed to add value: {res:?}");
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        core::write!(f, "{}", self.inner)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Value::null(),
        }
    }
}

#[cfg(test)]
/// Tests for the Value struct and its methods.
/// This module contains unit tests for the Value struct, including its methods and operations.
/// The value is a holder for a combination of a CoreValue representation and its actual type.
mod tests {
    use super::*;
    use crate::{
        assert_structural_eq, datex_list,
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        prelude::*,
        values::core_values::{
            endpoint::Endpoint,
            integer::{Integer, typed_integer::TypedInteger},
            list::List,
        },
    };
    use core::str::FromStr;
    use log::info;

    #[test]
    fn endpoint() {
        let endpoint = Value::from(Endpoint::from_str("@test").unwrap());
        assert_eq!(endpoint.to_string(), "@test");
    }

    #[test]
    fn new_addition_assignments() {
        let mut x = Value::from(42i8);
        let y = Value::from(27i8);

        x += y.clone();
        assert_eq!(x, Value::from(69i8));
    }

    #[test]
    fn new_additions() {
        let x = Value::from(42i8);
        let y = Value::from(27i8);

        let z = (x.clone() + y.clone()).unwrap();
        assert_eq!(z, Value::from(69i8));
    }

    #[test]
    fn list() {
        let mut a = List::from(vec![
            Value::from("42"),
            Value::from(42),
            Value::from(true),
        ]);

        a.push(Value::from(42));
        a.push(4);

        assert_eq!(a.len(), 5);

        let b = List::from(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(b.len(), 11);

        let c = datex_list![1, "test", 3, true, false];
        assert_eq!(c.len(), 5);
        assert_eq!(c[0], 1.into());
        assert_eq!(c[1], "test".into());
        assert_eq!(c[2], 3.into());
    }

    #[test]
    fn boolean() {
        let a = Value::from(true);
        let b = Value::from(false);
        let c = Value::from(false);
        assert_ne!(a, b);
        assert_eq!(b, c);

        let d = (!b.clone()).unwrap();
        assert_eq!(a, d);

        // We can't add two booleans together, so this should return None
        let a_plus_b = a.clone() + b.clone();
        assert!(a_plus_b.is_err());
    }

    #[test]
    fn equality_same_type() {
        let a = Value::from(42i8);
        let b = Value::from(42i8);
        let c = Value::from(27i8);

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);

        info!("{} === {}", a.clone(), b.clone());
        info!("{} !== {}", a.clone(), c.clone());
    }

    #[test]
    fn decimal() {
        let a = Value::from(42.1f32);
        let b = Value::from(27f32);

        let a_plus_b = (a.clone() + b.clone()).unwrap();
        assert_eq!(a_plus_b, Value::from(69.1f32));
        info!("{} + {} = {}", a.clone(), b.clone(), a_plus_b);
    }

    #[test]
    fn null() {
        let null_value = Value::null();
        assert_eq!(null_value.to_string(), "null");

        let maybe_value: Option<i8> = None;
        let null_value = Value::from(maybe_value);
        assert_eq!(null_value.to_string(), "null");
        assert!(null_value.is_null());
    }

    #[test]
    fn addition() {
        let a = Value::from(42i8);
        let b = Value::from(27i8);

        let a_plus_b = (a.clone() + b.clone()).unwrap();
        assert_eq!(a_plus_b, Value::from(69i8));
        info!("{} + {} = {}", a.clone(), b.clone(), a_plus_b);
    }

    #[test]
    fn string_concatenation() {
        let a = Value::from("Hello ");
        let b = Value::from(42i8);

        assert!(a.is_text());
        assert!(b.is_integer_i8());

        let a_plus_b = (a.clone() + b.clone()).unwrap();
        let b_plus_a = (b.clone() + a.clone()).unwrap();

        assert!(a_plus_b.is_text());
        assert!(b_plus_a.is_text());

        assert_eq!(a_plus_b, Value::from("Hello 42"));
        assert_eq!(b_plus_a, Value::from("42Hello "));

        info!("{} + {} = {}", a.clone(), b.clone(), a_plus_b);
        info!("{} + {} = {}", b.clone(), a.clone(), b_plus_a);
    }

    #[test]
    fn structural_equality() {
        let a = Value::from(42_i8);
        let b = Value::from(42_i32);
        assert!(a.is_integer_i8());

        assert_structural_eq!(a, b);

        assert_structural_eq!(
            Value::from(TypedInteger::I8(42)),
            Value::from(TypedInteger::U32(42)),
        );

        assert_structural_eq!(
            Value::from(42_i8),
            Value::from(Integer::from(42_i8))
        );
    }

    #[test]
    fn default_types() {
        let memory = &Memory::new();
        let val = Value::from(Integer::from(42));
        assert!(val.has_default_type(memory));

        let val = Value::from(42i8);
        assert!(val.has_default_type(memory));

        let val = Value {
            inner: CoreValue::Integer(Integer::from(42)),
            custom_type: Some(memory.get_core_type(CoreLibTypeId::Base(
                CoreLibBaseTypeId::Integer,
            ))),
        };

        assert!(val.has_default_type(memory));

        let val = Value {
            inner: CoreValue::Integer(Integer::from(42)),
            custom_type: Some(Type::Alias(
                TypeDefinition::ImplType(
                    Box::new(memory.get_core_type(CoreLibTypeId::Base(
                        CoreLibBaseTypeId::Integer,
                    ))),
                    vec![],
                )
                .into(),
            )),
        };

        assert!(!val.has_default_type(memory));
    }
}
