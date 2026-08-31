use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::callable::Callable;

impl Classification for Callable {}
impl HasClassification for Callable {}