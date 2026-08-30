use crate::preludes::derive::Text;
use crate::traits::convert_parts::{FromParts, IntoParts};

/// Default implementations - cannot be split into parts
impl IntoParts for Text {}
impl FromParts for Text {}