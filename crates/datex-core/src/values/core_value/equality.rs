use crate::{
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
    values::{
        core_value::CoreValue,
        core_values::{
            decimal::typed_decimal::TypedDecimal,
            integer::typed_integer::TypedInteger,
        },
    },
};

impl StructuralEq for CoreValue {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CoreValue::Boolean(a), CoreValue::Boolean(b)) => {
                a.structural_eq(b)
            }

            // Integers
            (CoreValue::Integer(a), CoreValue::Integer(b)) => {
                a.structural_eq(b)
            }

            // TypedIntegers
            (CoreValue::TypedInteger(a), CoreValue::TypedInteger(b)) => {
                a.structural_eq(b)
            }

            // Integers + TypedIntegers
            (CoreValue::Integer(a), CoreValue::TypedInteger(b))
            | (CoreValue::TypedInteger(b), CoreValue::Integer(a)) => {
                TypedInteger::IBig(a.clone()).structural_eq(b)
            }

            // Decimals
            (CoreValue::Decimal(a), CoreValue::Decimal(b)) => {
                a.structural_eq(b)
            }

            // TypedDecimals
            (CoreValue::TypedDecimal(a), CoreValue::TypedDecimal(b)) => {
                a.structural_eq(b)
            }

            // Decimal + TypedDecimal
            (CoreValue::Decimal(a), CoreValue::TypedDecimal(b))
            | (CoreValue::TypedDecimal(b), CoreValue::Decimal(a)) => {
                TypedDecimal::Decimal(a.clone()).structural_eq(b)
            }

            (CoreValue::Text(a), CoreValue::Text(b)) => a.structural_eq(b),
            (CoreValue::Null, CoreValue::Null) => true,
            (CoreValue::Endpoint(a), CoreValue::Endpoint(b)) => {
                a.structural_eq(b)
            }
            (CoreValue::List(a), CoreValue::List(b)) => a.structural_eq(b),
            (CoreValue::Map(a), CoreValue::Map(b)) => a.structural_eq(b),
            (CoreValue::Type(a), CoreValue::Type(b)) => a.structural_eq(b),
            (CoreValue::Callable(a), CoreValue::Callable(b)) => {
                a.structural_eq(b)
            }

            (CoreValue::Range(a), CoreValue::Range(b)) => {
                a.start.structural_eq(&b.start) && a.end.structural_eq(&b.end)
            }
            _ => false,
        }
    }
}

/// Value equality corresponds to partial equality for all values,
/// except for decimals, where partial equality is also given for NaN values and +0.0 and -0.0.
/// Therefore, we ValueEq is used instead for decimals
impl ValueEq for CoreValue {
    fn value_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CoreValue::Decimal(a), CoreValue::Decimal(b)) => a.value_eq(b),
            (CoreValue::TypedDecimal(a), CoreValue::TypedDecimal(b)) => {
                a.value_eq(b)
            }
            _ => self == other,
        }
    }
}
