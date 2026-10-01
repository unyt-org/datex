use crate::{
    traits::convert_parts::{FromParts, IntoParts, HasPartsKind},
    values::core_values::decimal::Decimal,
};

/// Default implementations - cannot be split into parts
impl IntoParts for Decimal {}
impl FromParts for Decimal {}
impl HasPartsKind for Decimal {}
