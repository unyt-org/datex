use crate::{
    traits::convert_parts::{FromParts, IntoParts, HasPartsKind},
    values::core_values::integer::Integer,
};

/// Default implementations - cannot be split into parts
impl IntoParts for Integer {}
impl FromParts for Integer {}
impl HasPartsKind for Integer {}
