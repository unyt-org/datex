use crate::traits::convert_parts::{FromParts, IntoParts};
use crate::values::core_values::callable::Callable;

/// Default implementations - cannot be split into parts
impl IntoParts for Callable {}
impl FromParts for Callable {}