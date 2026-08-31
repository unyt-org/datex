use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::native::DatexNativeBase;

impl<T> Classification for Vec<T>
where
    T: DatexNativeBase + 'static,
{}

impl<T> HasClassification for Vec<T>
where
    T: DatexNativeBase + 'static,
{}
