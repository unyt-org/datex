use crate::{
    random::RandomState,
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
        get_datex_type::GetDatexType,
    },
    values::core_values::native::DatexNativeBase,
};
use core::hash::Hash;
use indexmap::IndexMap;

/// `IndexMap<K, V, RandomState>` always implements [DatexNativeStructural].
impl<K: Eq + Hash + DatexNativeBase + 'static, V: DatexNativeBase + 'static>
    DatexNativeStructural for IndexMap<K, V, RandomState>
{
}

/// If `K` and `V` implement [DatexNativeOnlyStructural], then `IndexMap<K, V, RandomState>` also implements [DatexNativeOnlyStructural].
impl<
    K: DatexNativeOnlyStructural + GetDatexType + Eq + Hash,
    V: GetDatexType + DatexNativeOnlyStructural,
> DatexNativeOnlyStructural for IndexMap<K, V, RandomState>
{
}
