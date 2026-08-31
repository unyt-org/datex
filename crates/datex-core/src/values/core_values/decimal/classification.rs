use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::decimal::Decimal;

impl Classification for Decimal {}
impl HasClassification for Decimal {}