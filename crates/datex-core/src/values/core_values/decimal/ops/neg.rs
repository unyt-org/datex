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

impl Neg for Decimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Decimal::Finite(value) => Decimal::Finite(-value),
            Decimal::Zero => Decimal::NegZero,
            Decimal::NegZero => Decimal::Zero,
            Decimal::Infinity => Decimal::NegInfinity,
            Decimal::NegInfinity => Decimal::Infinity,
            Decimal::Nan => Decimal::Nan,
        }
    }
}

impl Neg for TypedDecimal {
    type Output = TypedDecimal;

    fn neg(self) -> Self::Output {
        match self {
            TypedDecimal::F32(value) => TypedDecimal::F32(value.neg()),
            TypedDecimal::F64(value) => TypedDecimal::F64(value.neg()),
            TypedDecimal::Decimal(value) => TypedDecimal::Decimal(value.neg()),
        }
    }
}
