use crate::{
    traits::convert_parts::{FromParts, IntoParts},
    values::core_values::endpoint::Endpoint,
};

/// Default implementations - cannot be split into parts
impl IntoParts for Endpoint {}
impl FromParts for Endpoint {}
