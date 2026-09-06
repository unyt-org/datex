use crate::{
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::core_values::callable::Callable,
};

impl Classification for Callable {}
impl StaticClassification for Callable {}
