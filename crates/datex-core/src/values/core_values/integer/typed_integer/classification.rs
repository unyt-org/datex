use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::integer::typed_integer::TypedInteger;

impl Classification for TypedInteger {}
impl StaticClassification for TypedInteger {}