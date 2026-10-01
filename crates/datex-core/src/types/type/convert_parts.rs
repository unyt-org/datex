use crate::{
    traits::convert_parts::{FromParts, IntoParts, HasPartsKind},
    types::r#type::Type,
};

/// Default implementations - cannot be split into parts
impl IntoParts for Type {}
impl FromParts for Type {}
impl HasPartsKind for Type {}
