use core::hash::Hash;
use indexmap::IndexMap;
use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::traits::get_datex_type::GetDatexType;
use crate::values::core_values::native::DatexNativeBase;
use crate::random::RandomState;

/// `IndexMap<K, V, RandomState>` always implements [DatexNativeStructural].
impl<K: Eq + Hash + DatexNativeBase + 'static, V: DatexNativeBase + 'static> DatexNativeStructural for IndexMap<K, V, RandomState> {}

/// If `K` and `V` implement [DatexNativeOnlyStructural], then `IndexMap<K, V, RandomState>` also implements [DatexNativeOnlyStructural].
impl<K: DatexNativeOnlyStructural + GetDatexType + Eq + Hash, V: GetDatexType + DatexNativeOnlyStructural> DatexNativeOnlyStructural for IndexMap<K, V, RandomState> {}