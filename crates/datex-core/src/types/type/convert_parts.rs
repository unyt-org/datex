use crate::{
    traits::convert_parts::{FromParts, IntoParts},
    types::r#type::Type,
};

/// Default implementations - cannot be split into parts
impl IntoParts for Type {}
impl FromParts for Type {}
