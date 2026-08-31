
use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::endpoint::Endpoint;

impl Classification for Endpoint {}
impl HasClassification for Endpoint {}