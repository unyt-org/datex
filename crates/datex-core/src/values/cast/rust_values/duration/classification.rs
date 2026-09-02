use core::time::Duration;
use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;

impl Classification for Duration {}
impl StaticClassification for Duration {}