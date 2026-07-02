use crate::{
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
    values::core_values::decimal::{Decimal, typed_decimal::TypedDecimal},
};

impl StructuralEq for TypedDecimal {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypedDecimal::F32(a), TypedDecimal::F32(b)) => {
                a.into_inner() == b.into_inner()
            }
            (TypedDecimal::F64(a), TypedDecimal::F64(b)) => {
                a.into_inner() == b.into_inner()
            }
            (TypedDecimal::F32(a), TypedDecimal::F64(b))
            | (TypedDecimal::F64(b), TypedDecimal::F32(a)) => {
                a.into_inner() as f64 == b.into_inner()
            }
            (TypedDecimal::Decimal(a), TypedDecimal::Decimal(b)) => {
                a.structural_eq(b)
            }
            (a, TypedDecimal::Decimal(b)) | (TypedDecimal::Decimal(b), a) => {
                match a {
                    TypedDecimal::F32(value) => {
                        b.structural_eq(&Decimal::from(value.into_inner()))
                    }
                    TypedDecimal::F64(value) => {
                        b.structural_eq(&Decimal::from(value.into_inner()))
                    }
                    _ => false,
                }
            }
        }
    }
}

impl ValueEq for TypedDecimal {
    fn value_eq(&self, other: &Self) -> bool {
        match (self, other) {
            // F32 and F32
            (TypedDecimal::F32(a), TypedDecimal::F32(b)) => {
                a.into_inner() == b.into_inner()
            }
            // F64 and F64
            (TypedDecimal::F64(a), TypedDecimal::F64(b)) => {
                a.into_inner() == b.into_inner()
            }
            // Big and Big
            (TypedDecimal::Decimal(a), TypedDecimal::Decimal(b)) => {
                a.value_eq(b)
            }
            _ => false,
        }
    }
}

impl PartialEq for TypedDecimal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // F32 and F32
            (TypedDecimal::F32(a), TypedDecimal::F32(b)) => {
                let a = a.into_inner();
                let b = b.into_inner();
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            // F64 and F64
            (TypedDecimal::F64(a), TypedDecimal::F64(b)) => {
                let a = a.into_inner();
                let b = b.into_inner();
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            // Big and Big
            (TypedDecimal::Decimal(a), TypedDecimal::Decimal(b)) => a == b,
            _ => false,
        }
    }
}
