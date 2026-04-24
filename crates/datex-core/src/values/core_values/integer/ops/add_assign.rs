use crate::{prelude::*, values::core_values::integer::Integer};
use core::{ops::AddAssign, result::Result};

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
impl AddAssign for TypedInteger {
    // FIXME #345 error handling / wrapping if out of bounds
    fn add_assign(&mut self, rhs: Self) {
        *self = TypedInteger::add(self.clone(), rhs).expect("Failed to add");
    }
}
