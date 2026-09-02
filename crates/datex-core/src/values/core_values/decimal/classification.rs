use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::decimal::Decimal;

impl Classification for Decimal {}
impl StaticClassification for Decimal {}