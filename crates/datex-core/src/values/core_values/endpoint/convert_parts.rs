use crate::{
    traits::convert_parts::{FromParts, IntoParts, WithPartsKind},
    values::core_values::endpoint::Endpoint,
};

/// Default implementations - cannot be split into parts
impl IntoParts for Endpoint {}
impl FromParts for Endpoint {}
impl WithPartsKind for Endpoint {}
