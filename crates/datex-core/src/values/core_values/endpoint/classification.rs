
use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::endpoint::Endpoint;

impl Classification for Endpoint {}
impl StaticClassification for Endpoint {}