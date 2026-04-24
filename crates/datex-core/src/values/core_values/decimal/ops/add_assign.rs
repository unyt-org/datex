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
    ops::{Add, AddAssign, Neg, Sub},
    str::FromStr,
};
use num::{BigInt, BigRational};
use num_enum::TryFromPrimitive;
use num_traits::{FromPrimitive, Zero};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
impl AddAssign for TypedDecimal {
    fn add_assign(&mut self, rhs: Self) {
        *self = TypedDecimal::add(self.clone(), rhs);
    }
}
