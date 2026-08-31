use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;

impl Classification for TypedDecimal {}
impl HasClassification for TypedDecimal {}