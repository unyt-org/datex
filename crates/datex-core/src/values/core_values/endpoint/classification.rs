use crate::{
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::core_values::endpoint::Endpoint,
};

impl Classification for Endpoint {}
impl StaticClassification for Endpoint {}
