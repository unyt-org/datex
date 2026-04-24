use crate::{
    libs::core::type_id::{
        CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId,
    },
    prelude::*,
    runtime::memory::Memory,
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
    types::{
        nominal_type_definition::NominalTypeDefinition,
        shared_container_containing_nominal_type::SharedContainerContainingNominalType,
        r#type::Type,
    },
    values::{
        core_value::CoreValue,
        core_values::{
            boolean::Boolean,
            callable::Callable,
            decimal::{
                Decimal,
                typed_decimal::{DecimalTypeVariant, TypedDecimal},
            },
            endpoint::Endpoint,
            integer::{
                Integer,
                typed_integer::{IntegerTypeVariant, TypedInteger},
            },
            list::List,
            map::Map,
            range::Range,
            text::Text,
        },
        value_container::{ValueContainer, error::ValueError},
    },
};
use core::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Neg, Not, Sub},
    result::Result,
};
use datex_macros_internal::FromCoreValue;

impl Add for CoreValue {
    type Output = Result<CoreValue, ValueError>;
    fn add(self, rhs: CoreValue) -> Self::Output {
        match (&self, &rhs) {
            // x + text or text + x (order does not matter)
            (CoreValue::Text(text), other) => {
                let other = other.cast_to_text();
                return Ok(CoreValue::Text(text + other));
            }
            (other, CoreValue::Text(text)) => {
                let other = other.cast_to_text();
                return Ok(CoreValue::Text(other + text));
            }

            // same type additions
            (CoreValue::TypedInteger(lhs), CoreValue::TypedInteger(rhs)) => {
                return Ok(CoreValue::TypedInteger(
                    (lhs + rhs).ok_or(ValueError::IntegerOverflow)?,
                ));
            }
            (CoreValue::Integer(lhs), CoreValue::Integer(rhs)) => {
                return Ok(CoreValue::Integer(lhs + rhs));
            }
            (CoreValue::TypedDecimal(lhs), CoreValue::TypedDecimal(rhs)) => {
                return Ok(CoreValue::TypedDecimal(lhs + rhs));
            }
            (CoreValue::Decimal(lhs), CoreValue::Decimal(rhs)) => {
                return Ok(CoreValue::Decimal(lhs + rhs));
            }

            _ => {}
        }

        // other cases
        match &self {
            // integer
            CoreValue::Integer(lhs) => match &rhs {
                CoreValue::TypedInteger(rhs) => {
                    Ok(CoreValue::Integer(lhs.clone() + rhs.as_integer()))
                }
                CoreValue::Decimal(_) => {
                    let integer = rhs
                        ._cast_to_integer_internal()
                        .ok_or(ValueError::InvalidOperation)?;
                    Ok(CoreValue::Integer(lhs.clone() + integer.as_integer()))
                }
                CoreValue::TypedDecimal(rhs) => {
                    let decimal = rhs.as_f64();
                    let integer = TypedInteger::from(decimal as i128);
                    Ok(CoreValue::Integer(lhs.clone() + integer.as_integer()))
                }
                _ => Err(ValueError::InvalidOperation),
            },

            // typed integer
            CoreValue::TypedInteger(lhs) => match &rhs {
                CoreValue::Integer(_rhs) => {
                    core::todo!(
                        "#317 TypedInteger + Integer not implemented yet"
                    );
                    //Ok(CoreValue::TypedInteger(lhs.as_integer() + rhs.clone()))
                }
                CoreValue::Decimal(_) => {
                    let integer = rhs
                        ._cast_to_integer_internal()
                        .ok_or(ValueError::InvalidOperation)?;
                    Ok(CoreValue::TypedInteger(
                        (lhs + &integer).ok_or(ValueError::IntegerOverflow)?,
                    ))
                }
                CoreValue::TypedDecimal(rhs) => {
                    let decimal = rhs.as_f64();
                    let integer = TypedInteger::from(decimal as i128);
                    Ok(CoreValue::TypedInteger(
                        (lhs + &integer).ok_or(ValueError::IntegerOverflow)?,
                    ))
                }
                _ => Err(ValueError::InvalidOperation),
            },

            // decimal
            CoreValue::Decimal(lhs) => match rhs {
                CoreValue::TypedDecimal(rhs) => {
                    Ok(CoreValue::Decimal(lhs + &Decimal::from(rhs)))
                }
                CoreValue::TypedInteger(rhs) => {
                    let decimal = Decimal::from(
                        rhs.as_i128().ok_or(ValueError::IntegerOverflow)?
                            as f64,
                    );
                    Ok(CoreValue::Decimal(lhs + &decimal))
                }
                CoreValue::Integer(rhs) => {
                    let decimal = Decimal::from(
                        rhs.as_i128().ok_or(ValueError::IntegerOverflow)?
                            as f64,
                    );
                    Ok(CoreValue::Decimal(lhs + &decimal))
                }
                _ => Err(ValueError::InvalidOperation),
            },

            // typed decimal
            CoreValue::TypedDecimal(lhs) => match rhs {
                CoreValue::Decimal(rhs) => Ok(CoreValue::TypedDecimal(
                    lhs + &TypedDecimal::Decimal(rhs),
                )),
                CoreValue::TypedInteger(rhs) => {
                    let decimal = TypedDecimal::from(
                        rhs.as_i128().ok_or(ValueError::IntegerOverflow)?
                            as f64,
                    );
                    Ok(CoreValue::TypedDecimal(lhs + &decimal))
                }
                CoreValue::Integer(rhs) => {
                    let decimal = TypedDecimal::from(
                        rhs.as_i128().ok_or(ValueError::IntegerOverflow)?
                            as f64,
                    );
                    Ok(CoreValue::TypedDecimal(lhs + &decimal))
                }
                _ => Err(ValueError::InvalidOperation),
            },

            _ => Err(ValueError::InvalidOperation),
        }
    }
}

impl Add for &CoreValue {
    type Output = Result<CoreValue, ValueError>;
    fn add(self, rhs: &CoreValue) -> Self::Output {
        CoreValue::add(self.clone(), rhs.clone())
    }
}
