use crate::{
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::core_values::integer::typed_integer::TypedInteger,
};

impl Classification for TypedInteger {}
impl StaticClassification for TypedInteger {}
