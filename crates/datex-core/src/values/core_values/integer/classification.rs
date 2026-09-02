use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::integer::Integer;

impl Classification for Integer {}
impl StaticClassification for Integer {}