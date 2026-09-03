use core::hash::Hash;
use indexmap::IndexMap;
use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::traits::get_datex_type::GetDatexType;
use crate::values::core_values::native::DatexNativeBase;

/// `IndexMap<K, V>` always implements [DatexNativeStructural].
impl<K: Eq + Hash + DatexNativeBase + 'static, V: DatexNativeBase + 'static> DatexNativeStructural for IndexMap<K, V> {}

/// If `K` and `V` implement [DatexNativeOnlyStructural], then `IndexMap<K, V>` also implements [DatexNativeOnlyStructural].
impl<K: DatexNativeOnlyStructural + GetDatexType + Eq + Hash, V: GetDatexType + DatexNativeOnlyStructural> DatexNativeOnlyStructural for IndexMap<K, V> {}