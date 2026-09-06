use crate::{
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::core_values::integer::Integer,
};

impl Classification for Integer {}
impl StaticClassification for Integer {}
