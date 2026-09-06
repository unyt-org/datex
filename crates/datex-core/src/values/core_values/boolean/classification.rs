use crate::{
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::core_values::boolean::Boolean,
};

impl Classification for Boolean {}
impl StaticClassification for Boolean {}
