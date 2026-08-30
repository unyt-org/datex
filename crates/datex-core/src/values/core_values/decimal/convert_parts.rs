use crate::traits::convert_parts::{FromParts, IntoParts};
use crate::values::core_values::decimal::Decimal;

/// Default implementations - cannot be split into parts
impl IntoParts for Decimal {}
impl FromParts for Decimal {}