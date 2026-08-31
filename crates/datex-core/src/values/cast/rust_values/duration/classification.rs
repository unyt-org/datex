use core::time::Duration;
use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;

impl Classification for Duration {}
impl HasClassification for Duration {}