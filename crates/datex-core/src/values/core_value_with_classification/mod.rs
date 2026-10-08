use core::cell::RefCell;

use crate::{
    preludes::derive::{
        AccessError, BorrowedValueKey, ConvertCoreValue, CoreValue,
        SharedReferencesCache, TaggedTypeDefinition, ValueClassification,
        ValueContainer, ValueTag,
    },
    types::{
        r#type::Type,
        type_definition::{
            TypeDefinition, impl_type::ImplMarkers,
            intersection::IntersectionTypeDefinition,
        },
    },
    values::value::Value,
};
mod value_access;

#[derive(Debug, Clone)]
pub struct CoreValueWithClassification {
    /// The inner representation of the value, which is a [CoreValue].
    pub inner: CoreValue,
    /// additional extensions for the value, including its entity type, impls, and tag
    pub classification: ValueClassification,
}

impl CoreValueWithClassification {
    pub fn is_uninitialized(&self) -> bool {
        matches!(&self.inner, CoreValue::Uninitialized)
    }

    pub fn is_null(&self) -> bool {
        matches!(self.inner, CoreValue::Null)
    }

    /// Tries to get a borrow of the current value as the specified type.
    /// Does not perform any type conversion.
    pub fn try_as<T>(&self) -> Option<&T>
    where
        T: ConvertCoreValue,
    {
        T::try_borrow_from_core_value(&self.inner).ok()
    }

    pub fn try_as_mut<T>(&mut self) -> Option<&mut T>
    where
        T: ConvertCoreValue,
    {
        T::try_borrow_mut_from_core_value(&mut self.inner).ok()
    }

    /// Tries to convert the current value into the specific specified type.
    /// Does not perform any type conversion.
    pub fn try_into_core_value_with_classification<T>(
        self,
    ) -> Result<T, CoreValueWithClassification>
    where
        T: ConvertCoreValue,
    {
        T::try_from_core_value(self.inner).map_err(|inner| {
            CoreValueWithClassification {
                inner,
                classification: self.classification,
            }
        })
    }

    /// Returns the actual current [TypeDefinition] of the value
    pub fn actual_type(&self) -> TypeDefinition {
        let mut types = Vec::<Type>::new();

        if let Some(entity_type) = &self.classification.entity_type {
            types.push(Type::Entity(entity_type.clone()));
        }

        if let Some(ValueTag { tag, is_empty }) = &self.classification.tag {
            types.push(
                TypeDefinition::TaggedType(TaggedTypeDefinition {
                    tag: tag.clone(),
                    ty: if *is_empty {
                        None
                    } else {
                        Some(Box::new(Type::core(
                            self.inner.default_core_type(),
                        )))
                    },
                })
                .into(),
            );
        }

        if !self.classification.impls.is_empty() {
            types.push(
                TypeDefinition::ImplMarkers(ImplMarkers::new(
                    self.classification.impls.clone(),
                ))
                .into(),
            );
        }

        if types.is_empty() {
            TypeDefinition::CoreType(self.inner.default_core_type())
        } else if types.len() == 1 {
            types.into_iter().next().unwrap().convert_to_definition()
        } else {
            TypeDefinition::Intersection(IntersectionTypeDefinition::new(types))
        }
    }

    /// Takes (removes) a property from the value if applicable (e.g. for map and structs)
    pub fn try_take_property<'a>(
        &mut self,
        key: impl Into<BorrowedValueKey<'a>>,
        _cache: &RefCell<SharedReferencesCache>,
    ) -> Result<ValueContainer, AccessError> {
        // TODO
        match self.inner {
            CoreValue::Map(ref mut map) => {
                // If the value is a map, get the property
                Ok(map.try_delete(key)?)
            }
            CoreValue::List(ref mut list) => {
                if let Some(index) = key.into().try_as_index() {
                    Ok(list.try_delete(index)?)
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
                    list.try_delete(index)?;
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
                    if let ValueContainer::Local(Value::Core(v)) = &val
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
