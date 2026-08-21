use core::hash::Hash;

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

impl PartialEq for CoreValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CoreValue::Uninitialized, CoreValue::Uninitialized) => true,
            (CoreValue::Null, CoreValue::Null) => true,
            (CoreValue::Boolean(b1), CoreValue::Boolean(b2)) => b1 == b2,
            (CoreValue::Integer(i1), CoreValue::Integer(i2)) => i1 == i2,
            (CoreValue::TypedInteger(ti1), CoreValue::TypedInteger(ti2)) => {
                ti1 == ti2
            }
            (CoreValue::Decimal(d1), CoreValue::Decimal(d2)) => d1 == d2,
            (CoreValue::TypedDecimal(td1), CoreValue::TypedDecimal(td2)) => {
                td1 == td2
            }
            (CoreValue::Text(t1), CoreValue::Text(t2)) => t1 == t2,
            (CoreValue::Endpoint(e1), CoreValue::Endpoint(e2)) => e1 == e2,
            (CoreValue::List(l1), CoreValue::List(l2)) => l1 == l2,
            (CoreValue::Map(m1), CoreValue::Map(m2)) => m1 == m2,
            (CoreValue::Type(t1), CoreValue::Type(t2)) => t1 == t2,
            (
                CoreValue::EntityTypeDefinition(etd1),
                CoreValue::EntityTypeDefinition(etd2),
            ) => etd1 == etd2,
            (CoreValue::Callable(c1), CoreValue::Callable(c2)) => c1 == c2,
            (CoreValue::Range(r1), CoreValue::Range(r2)) => r1 == r2,
            (CoreValue::Box(b1), CoreValue::Box(b2)) => *b1 == *b2,
            (CoreValue::Native(n1), CoreValue::Native(n2)) => {
                n1.value.dyn_eq(&*n2.value)
            }
            _ => false, // TODO: compare with native
        }
    }
}

impl Eq for CoreValue {}

impl Hash for CoreValue {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            CoreValue::Uninitialized => state.write_u8(0),
            CoreValue::Null => state.write_u8(1),
            CoreValue::Boolean(b) => b.hash(state),
            CoreValue::Integer(i) => i.hash(state),
            CoreValue::TypedInteger(ti) => ti.hash(state),
            CoreValue::Decimal(d) => d.hash(state),
            CoreValue::TypedDecimal(td) => td.hash(state),
            CoreValue::Text(t) => t.hash(state),
            CoreValue::Endpoint(e) => e.hash(state),
            CoreValue::List(l) => l.hash(state),
            CoreValue::Map(m) => m.hash(state),
            CoreValue::Type(t) => t.hash(state),
            CoreValue::EntityTypeDefinition(etd) => etd.hash(state),
            CoreValue::Callable(c) => c.hash(state),
            CoreValue::Range(r) => r.hash(state),
            CoreValue::Box(b) => b.hash(state),
            CoreValue::Native(e) => {
                todo!("Hashing for native values is not implemented yet")
            }
        }
    }
}
