use crate::{
    traits::convert_parts::{FromParts, IntoParts, WithPartsKind},
    values::core_values::boolean::Boolean,
};

/// Default implementations - cannot be split into parts
impl IntoParts for Boolean {}
impl FromParts for Boolean {}
impl WithPartsKind for Boolean {}
