use crate::{
    traits::convert_parts::{FromParts, IntoParts, WithPartsKind},
    values::core_values::integer::Integer,
};

/// Default implementations - cannot be split into parts
impl IntoParts for Integer {}
impl FromParts for Integer {}
impl WithPartsKind for Integer {}
