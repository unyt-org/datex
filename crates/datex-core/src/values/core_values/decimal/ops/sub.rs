use crate::{
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
    values::core_values::{
        decimal::{Decimal, typed_decimal::TypedDecimal},
        error::NumberParseError,
    },
};
use bigdecimal::BigDecimal;
use binrw::{
    BinRead, BinReaderExt, BinResult, BinWrite, Endian,
    io::{Read, Seek, Write},
};
use core::{
    cmp::Ordering,
    fmt::Display,
    hash::Hash,
    ops::{Add, Neg, Sub},
    str::FromStr,
};
use num::{BigInt, BigRational};
use num_enum::TryFromPrimitive;
use num_traits::{FromPrimitive, Zero};
use serde::{Deserialize, Serialize};

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Sub for &Decimal {
    type Output = Decimal;

    fn sub(self, rhs: Self) -> Self::Output {
        // FIXME #335: Avoid cloning, as sub should be applicable for refs only
        Decimal::sub(self.clone(), rhs.clone())
    }
}

impl Sub for TypedDecimal {
    type Output = TypedDecimal;

    fn sub(self, rhs: Self) -> Self::Output {
        // negate rhs
        let negated_rhs = match rhs {
            TypedDecimal::F32(value) => TypedDecimal::F32(value.neg()),
            TypedDecimal::F64(value) => TypedDecimal::F64(value.neg()),
            TypedDecimal::Decimal(value) => TypedDecimal::Decimal(value.neg()),
        };

        // perform addition with negated rhs
        TypedDecimal::add(self, negated_rhs)
    }
}
impl Sub for &TypedDecimal {
    type Output = TypedDecimal;

    fn sub(self, rhs: Self) -> Self::Output {
        // FIXME #340: Avoid cloning, as sub should be applicable for refs only
        TypedDecimal::sub(self.clone(), rhs.clone())
    }
}
