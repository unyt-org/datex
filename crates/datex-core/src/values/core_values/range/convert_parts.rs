use crate::{
    traits::convert_parts::{FromParts, IntoParts, HasPartsKind},
    values::core_values::range::Range,
};

impl FromParts for Range {}
impl IntoParts for Range {}
impl HasPartsKind for Range {}
