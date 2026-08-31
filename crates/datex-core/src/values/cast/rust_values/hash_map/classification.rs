use crate::collections::HashMap;
use core::hash::Hash;
use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::native::DatexNativeBase;

impl<K, V> HasClassification for HashMap<K, V>
where
    K: DatexNativeBase + Eq + Hash + 'static,
    V: DatexNativeBase + 'static,
{}

impl<K, V> Classification for HashMap<K, V>
where
    K: DatexNativeBase + Eq + Hash + 'static,
    V: DatexNativeBase + 'static,
{}