use crate::{
    preludes::derive::Text,
    traits::convert_parts::{FromParts, IntoParts, WithPartsKind},
};

/// Default implementations - cannot be split into parts
impl IntoParts for Text {}
impl FromParts for Text {}
impl WithPartsKind for Text {}
