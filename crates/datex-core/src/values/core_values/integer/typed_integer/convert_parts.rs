use crate::traits::convert_parts::{FromParts, IntoParts};
use crate::values::core_values::integer::typed_integer::TypedInteger;

/// Default implementations - cannot be split into parts
impl IntoParts for TypedInteger {}
impl FromParts for TypedInteger {}