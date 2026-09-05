use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;

impl Classification for TypedDecimal {}
impl StaticClassification for TypedDecimal {}