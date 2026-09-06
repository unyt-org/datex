use crate::{
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::core_values::range::Range,
};

impl Classification for Range {}
impl StaticClassification for Range {}
