use crate::{
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::core_values::text::Text,
};

impl Classification for Text {}
impl StaticClassification for Text {}
