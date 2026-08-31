use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::integer::typed_integer::TypedInteger;

impl Classification for TypedInteger {}
impl HasClassification for TypedInteger {}