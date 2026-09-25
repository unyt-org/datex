use crate::{
    preludes::derive::DatexNative,
    random::RandomState,
    traits::local_child_path_resolver::LocalChildPathResolver,
    value_updates::errors::UpdateError,
    values::{
        core_values::native::{DatexNativeBase, DatexNativeOps},
        value::Value,
        value_container::value_key::ValueKey,
    },
};
use core::{any::Any, hash::Hash};
use indexmap::IndexMap;

impl<K, V> DatexNative for IndexMap<K, V, RandomState>
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
impl<K, V> DatexNativeOps for IndexMap<K, V, RandomState>
where
    K: DatexNativeBase + Eq + Hash + 'static,
    V: DatexNativeBase + 'static,
{
}

impl<K, V> LocalChildPathResolver for IndexMap<K, V, RandomState>
where
    K: DatexNativeBase + Eq + Hash + 'static,
    V: DatexNativeBase + 'static,
{
    fn resolve_value_for_path(
        &mut self,
        first: &ValueKey,
        remaining_path: &[ValueKey],
    ) -> Result<&mut Value, UpdateError> {
        todo!()
    }
}
