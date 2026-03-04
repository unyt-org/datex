use crate::{
    dif::{
        DIFConvertible, representation::DIFValueRepresentation,
        r#type::DIFTypeDefinition,
    },
    libs::core::CoreLibPointerId,
    runtime::memory::Memory,
    types::definition::TypeDefinition,
    values::{
        core_value::CoreValue,
        core_values::{
            decimal::typed_decimal::{DecimalTypeVariant, TypedDecimal},
            integer::typed_integer::TypedInteger,
            map::{BorrowedMapKey, Map},
        },
        value::Value,
        value_container::ValueContainer,
    },
};
use core::{cell::RefCell, result::Result};
use serde::{Deserialize, Serialize};

use crate::{prelude::*, shared_values::pointer_address::PointerAddress};

#[derive(Debug)]
pub struct DIFReferenceNotFoundError;

/// Represents a value in the Datex Interface Format (DIF).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DIFValue {
    pub value: DIFValueRepresentation,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub ty: Option<DIFTypeDefinition>,
}
impl DIFConvertible for DIFValue {}

impl DIFValue {
    /// Converts the DIFValue into a Value, resolving references using the provided memory.
    /// Returns an error if a reference cannot be resolved.
    pub fn to_value(
        &self,
        memory: &RefCell<Memory>,
    ) -> Result<Value, DIFReferenceNotFoundError> {
        Ok(if let Some(ty) = &self.ty {
            self.value.to_value_with_type(ty, memory)?
        } else {
            self.value.to_default_value(memory)?
        })
    }
}

impl DIFValue {
    pub fn new(
        value: DIFValueRepresentation,
        ty: Option<impl Into<DIFTypeDefinition>>,
    ) -> Self {
        DIFValue {
            value,
            ty: ty.map(Into::into),
        }
    }
    pub fn as_container(&self) -> DIFValueContainer {
        DIFValueContainer::from(self.clone())
    }
}

impl From<DIFValueRepresentation> for DIFValue {
    fn from(value: DIFValueRepresentation) -> Self {
        DIFValue { value, ty: None }
    }
}

/// Holder for either a value or a reference to a value in DIF
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DIFValueContainer {
    Value(DIFValue),
    Reference(PointerAddress),
}
impl DIFConvertible for DIFValueContainer {}

impl DIFValueContainer {
    /// Converts the DIFValueContainer into a ValueContainer, resolving references using the provided memory.
    /// Returns an error if a reference cannot be resolved.
    pub fn to_value_container(
        &self,
        memory: &RefCell<Memory>,
    ) -> Result<ValueContainer, DIFReferenceNotFoundError> {
        Ok(match self {
            DIFValueContainer::Reference(address) => ValueContainer::Shared(
                memory
                    .borrow()
                    .get_reference(address)
                    .ok_or(DIFReferenceNotFoundError)?
                    .clone(),
            ),
            DIFValueContainer::Value(v) => {
                ValueContainer::Local(v.to_value(memory)?)
            }
        })
    }
}

impl From<DIFValue> for DIFValueContainer {
    fn from(value: DIFValue) -> Self {
        DIFValueContainer::Value(value)
    }
}
impl From<&DIFValue> for DIFValueContainer {
    fn from(value: &DIFValue) -> Self {
        DIFValueContainer::Value(value.clone())
    }
}
impl From<PointerAddress> for DIFValueContainer {
    fn from(ptr: PointerAddress) -> Self {
        DIFValueContainer::Reference(ptr)
    }
}

impl DIFValueContainer {
    pub fn from_value_container(value_container: &ValueContainer) -> Self {
        match value_container {
            ValueContainer::Shared(reference) => {
                DIFValueContainer::Reference(reference.pointer_address())
            }
            ValueContainer::Local(value) => {
                DIFValueContainer::Value(DIFValue::from_value(value))
            }
        }
    }
}

impl DIFValue {
    fn from_value(value: &Value) -> Self {
        let core_value = &value.inner;

        let mut is_empty_map = false;

        let dif_core_value = match core_value {
            CoreValue::Type(_ty) => {
                core::todo!("#382 Type value not supported in DIF")
            }
            CoreValue::Callable(_callable) => {
                core::todo!("#616 Callable value not yet supported in DIF")
            }
            CoreValue::Null => DIFValueRepresentation::Null,
            CoreValue::Boolean(bool) => DIFValueRepresentation::Boolean(bool.0),
            CoreValue::Integer(integer) => {
                // TODO #383: optimize this and pass as integer if in range
                DIFValueRepresentation::String(integer.to_string())
            }
            CoreValue::TypedInteger(integer) => {
                match integer {
                    TypedInteger::I8(i) => {
                        DIFValueRepresentation::Number(*i as f64)
                    }
                    TypedInteger::U8(u) => {
                        DIFValueRepresentation::Number(*u as f64)
                    }
                    TypedInteger::I16(i) => {
                        DIFValueRepresentation::Number(*i as f64)
                    }
                    TypedInteger::U16(u) => {
                        DIFValueRepresentation::Number(*u as f64)
                    }
                    TypedInteger::I32(i) => {
                        DIFValueRepresentation::Number(*i as f64)
                    }
                    TypedInteger::U32(u) => {
                        DIFValueRepresentation::Number(*u as f64)
                    }
                    // i64 and above are serialized as strings in DIF
                    TypedInteger::I64(i) => {
                        DIFValueRepresentation::String(i.to_string())
                    }
                    TypedInteger::U64(u) => {
                        DIFValueRepresentation::String(u.to_string())
                    }
                    TypedInteger::I128(i) => {
                        DIFValueRepresentation::String(i.to_string())
                    }
                    TypedInteger::U128(u) => {
                        DIFValueRepresentation::String(u.to_string())
                    }
                    TypedInteger::IBig(i) => {
                        DIFValueRepresentation::String(i.to_string())
                    }
                }
            }
            CoreValue::Range(_range) => {
                core::todo!("#740 Range value not yet supported in DIF")
            }
            CoreValue::Decimal(decimal) => {
                // TODO #384: optimize this and pass as decimal if in range
                DIFValueRepresentation::String(decimal.to_string())
            }
            CoreValue::TypedDecimal(decimal) => match decimal {
                TypedDecimal::F32(f) => {
                    DIFValueRepresentation::Number(f.0 as f64)
                }
                TypedDecimal::F64(f) => DIFValueRepresentation::Number(f.0),
                TypedDecimal::Decimal(bd) => {
                    DIFValueRepresentation::String(bd.to_string())
                }
            },
            CoreValue::Text(text) => {
                DIFValueRepresentation::String(text.0.clone())
            }
            CoreValue::Endpoint(endpoint) => {
                DIFValueRepresentation::String(endpoint.to_string())
            }
            CoreValue::List(list) => DIFValueRepresentation::Array(
                list.iter()
                    .map(|v| DIFValueContainer::from_value_container(v))
                    .collect(),
            ),
            CoreValue::Map(map) => match map {
                Map::StructuralWithStringKeys(entries) => {
                    DIFValueRepresentation::Object(
                        entries
                            .iter()
                            .map(|(k, v)| {
                                (
                                    k.clone(),
                                    DIFValueContainer::from_value_container(v),
                                )
                            })
                            .collect(),
                    )
                }
                _ => {
                    if map.is_empty() {
                        is_empty_map = true;
                    };

                    DIFValueRepresentation::Map(
                        map.iter()
                            .map(|(k, v)| {
                                (
                                    match k {
                                        BorrowedMapKey::Text(text_key) => {
                                            DIFValueContainer::Value(
                                                DIFValueRepresentation::String(
                                                    text_key.to_string(),
                                                )
                                                    .into(),
                                            )
                                        }
                                        _ => {
                                            DIFValueContainer::from_value_container(
                                                &ValueContainer::from(k),
                                            )
                                        }
                                    },
                                    DIFValueContainer::from_value_container(
                                        v
                                    ),
                                )
                            })
                            .collect(),
                    )
                }
            },
        };

        DIFValue {
            value: dif_core_value,
            ty: get_type_if_non_default(&value.actual_type, is_empty_map),
        }
    }
}

/// Returns the type if it is not the default type for the value, None otherwise
/// We treat the following types as default:
/// - boolean
/// - text
/// - null
/// - decimal (f64)
/// - List
/// - Map (if not empty, otherwise we cannot distinguish between empty map and empty list since both are represented as [] in DIF)
fn get_type_if_non_default(
    type_definition: &TypeDefinition,
    is_empty_map: bool,
) -> Option<DIFTypeDefinition> {
    match type_definition {
        TypeDefinition::SharedReference(inner) => {
            if let Ok(address) =
                CoreLibPointerId::try_from(&inner.borrow().pointer.address())
                && (core::matches!(
                        address,
                        CoreLibPointerId::Decimal(Some(DecimalTypeVariant::F64))
                            | CoreLibPointerId::Boolean
                            | CoreLibPointerId::Text
                            | CoreLibPointerId::List
                            | CoreLibPointerId::Null
                    ) ||
                    // map is default only if not empty
                    (core::matches!(address, CoreLibPointerId::Map) && !is_empty_map))
            {
                None
            } else {
                Some(DIFTypeDefinition::from_type_definition(type_definition))
            }
        }
        _ => Some(DIFTypeDefinition::from_type_definition(type_definition)),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dif::{DIFConvertible, r#type::DIFTypeDefinition, value::DIFValue},
        libs::core::CoreLibPointerId,
        runtime::memory::Memory,
        values::{
            core_values::{
                endpoint::Endpoint, integer::typed_integer::IntegerTypeVariant,
                map::Map,
            },
            value_container::ValueContainer,
        },
    };

    use crate::{prelude::*, values::value::Value};
    use core::cell::RefCell;

    fn get_mock_memory() -> RefCell<Memory> {
        RefCell::new(Memory::new(Endpoint::default()))
    }

    #[test]
    fn default_type() {
        let dif = DIFValue::from_value(&Value::from(true));
        assert!(dif.ty.is_none());

        let dif = DIFValue::from_value(&Value::from("hello"));
        assert!(dif.ty.is_none());

        let dif = DIFValue::from_value(&Value::null());
        assert!(dif.ty.is_none());

        let dif = DIFValue::from_value(&Value::from(3.5f64));
        assert!(dif.ty.is_none());

        let dif = DIFValue::from_value(&Value::from(vec![
            Value::from(1),
            Value::from(2),
            Value::from(3),
        ]));
        assert!(dif.ty.is_none());

        let dif = DIFValue::from_value(&Value::from(Map::from(vec![
            ("a".to_string(), ValueContainer::from(1)),
            ("b".to_string(), ValueContainer::from(2)),
        ])));
        assert!(dif.ty.is_none());
    }

    #[test]
    fn non_default_type() {
        let dif = DIFValue::from_value(&Value::from(123u16));
        assert!(dif.ty.is_some());
        if let DIFTypeDefinition::Reference(reference) = dif.ty.unwrap() {
            assert_eq!(
                reference,
                CoreLibPointerId::Integer(Some(IntegerTypeVariant::U16)).into()
            );
        } else {
            core::panic!("Expected reference type");
        }

        let dif = DIFValue::from_value(&Value::from(123i64));
        assert!(dif.ty.is_some());
        if let DIFTypeDefinition::Reference(reference) = dif.ty.unwrap() {
            assert_eq!(
                reference,
                CoreLibPointerId::Integer(Some(IntegerTypeVariant::I64)).into()
            );
        } else {
            core::panic!("Expected reference type");
        }
    }

    #[test]
    fn serde_dif_value() {
        let dif = DIFValue::from_value(&Value::from("Hello, world!"));
        let serialized = dif.as_json();
        let deserialized = DIFValue::from_json(&serialized);
        assert_eq!(dif, deserialized);
    }
}
