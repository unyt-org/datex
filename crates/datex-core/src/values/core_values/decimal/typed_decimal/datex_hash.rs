use crate::{
    traits::datex_hash::impl_datex_hash,
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

impl_datex_hash!(TypedDecimal);
