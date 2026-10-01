use crate::{
    traits::convert_parts::{FromParts, IntoParts, HasPartsKind},
    values::core_values::integer::typed_integer::TypedInteger,
};

/// Default implementations - cannot be split into parts
impl IntoParts for TypedInteger {}
impl FromParts for TypedInteger {}
impl HasPartsKind for TypedInteger {}
