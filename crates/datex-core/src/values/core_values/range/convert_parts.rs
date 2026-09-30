use crate::{
    traits::convert_parts::{FromParts, IntoParts, WithPartsKind},
    values::core_values::range::Range,
};

impl FromParts for Range {}
impl IntoParts for Range {}
impl WithPartsKind for Range {}
