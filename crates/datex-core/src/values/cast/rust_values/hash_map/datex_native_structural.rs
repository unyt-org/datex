use core::hash::Hash;
use crate::collections::HashMap;
use crate::preludes::derive::DatexNative;
use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::traits::get_datex_type::GetDatexType;

/// If `T` implements [DatexNativeStructural], then `HashMap<K, V>` also implements [DatexNativeStructural].
impl<K: Eq + Hash + DatexNative + GetDatexType, V: DatexNative + GetDatexType> DatexNativeStructural for HashMap<K, V> {}

/// If `T` implements [DatexNativeOnlyStructural], then `HashMap<K, V>` also implements [DatexNativeOnlyStructural].
impl<K: DatexNativeOnlyStructural + GetDatexType + Eq + Hash, V: GetDatexType + DatexNativeOnlyStructural> DatexNativeOnlyStructural for HashMap<K, V> {}