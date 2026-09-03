use core::any::Any;
use core::hash::Hash;
use indexmap::IndexMap;
use crate::preludes::derive::{CoreValue, DatexNative, SharedReferencesCache, Type};
use crate::values::core_values::native::DatexNativeBase;

impl<K, V> DatexNative for IndexMap<K, V>
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