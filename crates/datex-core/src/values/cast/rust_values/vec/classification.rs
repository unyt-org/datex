use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::native::DatexNativeBase;
use crate::prelude::*;

impl<T> Classification for Vec<T>
where
    T: DatexNativeBase + 'static,
{}

impl<T> StaticClassification for Vec<T>
where
    T: DatexNativeBase + 'static,
{}
