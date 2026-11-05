use crate::{
    prelude::*,
    traits::{datex_hash::impl_datex_hash, value_access::ValueAccess},
};

mod convert_value;
mod to_instructions;
pub mod try_clone;

pub mod serde_dif;
#[cfg(feature = "ast")]
mod to_datex_expression_data;

impl ValueAccess for String {}

impl_datex_hash!(String);
