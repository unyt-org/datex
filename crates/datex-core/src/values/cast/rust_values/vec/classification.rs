use crate::{
    prelude::*,
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::core_values::native::DatexNativeBase,
};

impl<T> Classification for Vec<T> where T: DatexNativeBase + 'static {}

impl<T> StaticClassification for Vec<T> where T: DatexNativeBase + 'static {}
