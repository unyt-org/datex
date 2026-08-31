use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::text::Text;

impl Classification for Text {}
impl HasClassification for Text {}