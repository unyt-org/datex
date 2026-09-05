use core::hash::Hash;
use indexmap::IndexMap;
use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::native::DatexNativeBase;
use crate::random::RandomState;

impl<K, V> StaticClassification for IndexMap<K, V, RandomState>
where
    K: DatexNativeBase + Eq + Hash + 'static,
    V: DatexNativeBase + 'static,
{}

impl<K, V> Classification for IndexMap<K, V, RandomState>
where
    K: DatexNativeBase + Eq + Hash + 'static,
    V: DatexNativeBase + 'static,
{}