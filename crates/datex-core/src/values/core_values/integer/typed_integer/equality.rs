use crate::{
    traits::structural_eq::StructuralEq,
    values::core_values::integer::typed_integer::TypedInteger,
};

// FIXME #343 discuss on structural vs partial equality for integers
impl StructuralEq for TypedInteger {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypedInteger::I8(v1), TypedInteger::I8(v2)) => v1 == v2,
            (TypedInteger::I16(v1), TypedInteger::I16(v2)) => v1 == v2,
            (TypedInteger::I32(v1), TypedInteger::I32(v2)) => v1 == v2,
            (TypedInteger::I64(v1), TypedInteger::I64(v2)) => v1 == v2,
            (TypedInteger::I128(v1), TypedInteger::I128(v2)) => v1 == v2,
            (TypedInteger::U8(v1), TypedInteger::U8(v2)) => v1 == v2,
            (TypedInteger::U16(v1), TypedInteger::U16(v2)) => v1 == v2,
            (TypedInteger::U32(v1), TypedInteger::U32(v2)) => v1 == v2,
            (TypedInteger::U64(v1), TypedInteger::U64(v2)) => v1 == v2,
            (TypedInteger::U128(v1), TypedInteger::U128(v2)) => v1 == v2,
            (TypedInteger::IBig(i1), TypedInteger::IBig(i2)) => i1 == i2,
            (a, b) => a.as_integer() == b.as_integer(),
        }
    }
}

impl PartialEq for TypedInteger {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypedInteger::I8(v1), TypedInteger::I8(v2)) => v1 == v2,
            (TypedInteger::I16(v1), TypedInteger::I16(v2)) => v1 == v2,
            (TypedInteger::I32(v1), TypedInteger::I32(v2)) => v1 == v2,
            (TypedInteger::I64(v1), TypedInteger::I64(v2)) => v1 == v2,
            (TypedInteger::I128(v1), TypedInteger::I128(v2)) => v1 == v2,
            (TypedInteger::U8(v1), TypedInteger::U8(v2)) => v1 == v2,
            (TypedInteger::U16(v1), TypedInteger::U16(v2)) => v1 == v2,
            (TypedInteger::U32(v1), TypedInteger::U32(v2)) => v1 == v2,
            (TypedInteger::U64(v1), TypedInteger::U64(v2)) => v1 == v2,
            (TypedInteger::U128(v1), TypedInteger::U128(v2)) => v1 == v2,
            (TypedInteger::IBig(i1), TypedInteger::IBig(i2)) => i1 == i2,
            _ => false,
        }
    }
}
