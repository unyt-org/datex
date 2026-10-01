use crate::{
    preludes::derive::Text,
    traits::convert_parts::{FromParts, IntoParts, HasPartsKind},
};

/// Default implementations - cannot be split into parts
impl IntoParts for Text {}
impl FromParts for Text {}
impl HasPartsKind for Text {}
