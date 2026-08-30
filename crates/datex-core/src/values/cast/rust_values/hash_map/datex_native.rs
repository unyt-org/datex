use core::any::Any;
use core::hash::Hash;
use crate::collections::HashMap;
use crate::preludes::derive::{DatexNative, SharedReferencesCache, Type};
use crate::traits::get_datex_type::GetDatexType;

impl<K, V> DatexNative for HashMap<K, V>
where
    K: DatexNative + GetDatexType + Eq + Hash,
    V: DatexNative + GetDatexType,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn value_datex_type(&self, cache: &mut SharedReferencesCache) -> Type {
        <Self as GetDatexType>::datex_type(cache)
    }
}
