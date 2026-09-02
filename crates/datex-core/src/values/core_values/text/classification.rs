use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::text::Text;

impl Classification for Text {}
impl StaticClassification for Text {}