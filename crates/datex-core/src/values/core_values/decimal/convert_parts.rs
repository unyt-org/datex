use crate::{
    traits::convert_parts::{FromParts, IntoParts, WithPartsKind},
    values::core_values::decimal::Decimal,
};

/// Default implementations - cannot be split into parts
impl IntoParts for Decimal {}
impl FromParts for Decimal {}
impl WithPartsKind for Decimal {}
