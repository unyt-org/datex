use crate::{
    traits::convert_parts::{FromParts, IntoParts},
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

/// Default implementations - cannot be split into parts
impl IntoParts for TypedDecimal {}
impl FromParts for TypedDecimal {}
