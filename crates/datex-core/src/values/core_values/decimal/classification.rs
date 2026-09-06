use crate::{
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::core_values::decimal::Decimal,
};

impl Classification for Decimal {}
impl StaticClassification for Decimal {}
