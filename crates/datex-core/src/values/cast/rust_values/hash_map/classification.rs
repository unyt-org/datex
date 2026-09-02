use crate::collections::HashMap;
use core::hash::Hash;
use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::native::DatexNativeBase;

impl<K, V> StaticClassification for HashMap<K, V>
where
    K: DatexNativeBase + Eq + Hash + 'static,
    V: DatexNativeBase + 'static,
{}

impl<K, V> Classification for HashMap<K, V>
where
    K: DatexNativeBase + Eq + Hash + 'static,
    V: DatexNativeBase + 'static,
{}