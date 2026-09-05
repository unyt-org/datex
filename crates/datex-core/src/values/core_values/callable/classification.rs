use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::callable::Callable;

impl Classification for Callable {}
impl StaticClassification for Callable {}