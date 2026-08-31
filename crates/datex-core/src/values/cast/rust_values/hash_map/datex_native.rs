use core::any::Any;
use core::hash::Hash;
use crate::collections::HashMap;
use crate::preludes::derive::{CoreValue, DatexNative, SharedReferencesCache, Type};
use crate::values::core_values::native::DatexNativeBase;
use crate::values::value::value_classification::ValueClassification;

impl<K, V> DatexNative for HashMap<K, V>
where
    K: DatexNativeBase + Eq + Hash + 'static,
    V: DatexNativeBase + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}