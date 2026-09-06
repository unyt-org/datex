use crate::{
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

impl Classification for TypedDecimal {}
impl StaticClassification for TypedDecimal {}
