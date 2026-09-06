use crate::traits::{
    classification::Classification, static_classification::StaticClassification,
};
use core::time::Duration;

impl Classification for Duration {}
impl StaticClassification for Duration {}
