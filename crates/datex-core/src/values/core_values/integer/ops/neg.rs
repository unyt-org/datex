use crate::{
    prelude::*,
    values::{
        core_values::integer::Integer, value_container::error::ValueError,
    },
};
use core::result::Result;

use crate::{
    traits::structural_eq::StructuralEq,
    values::core_values::{
        error::NumberParseError, integer::typed_integer::TypedInteger,
    },
};
use binrw::{
    BinRead, BinReaderExt, BinResult, BinWrite, Endian,
    io::{Read, Seek, Write},
};
use core::{
    fmt::Display,
    hash::Hash,
    ops::{Add, Neg, Sub},
    str::FromStr,
};
use num::{BigInt, Num};
use num_integer::Integer as NumInteger;
use num_traits::ToPrimitive;
use serde::Deserialize;

impl Neg for Integer {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Integer(-self.0)
    }
}

// FIXME #347 shall we allow negation of unsigned integers and wrap around?
impl Neg for TypedInteger {
    type Output = Result<TypedInteger, ValueError>;

    fn neg(self) -> Self::Output {
        match self {
            TypedInteger::I8(v) => Ok(TypedInteger::I8(v.neg())),
            TypedInteger::I16(v) => Ok(TypedInteger::I16(v.neg())),
            TypedInteger::I32(v) => Ok(TypedInteger::I32(v.neg())),
            TypedInteger::I64(v) => Ok(TypedInteger::I64(v.neg())),
            TypedInteger::I128(v) => Ok(TypedInteger::I128(v.neg())),
            TypedInteger::IBig(v) => Ok(TypedInteger::IBig(v.neg())),
            _ => Err(ValueError::InvalidOperation),
        }
    }
}
