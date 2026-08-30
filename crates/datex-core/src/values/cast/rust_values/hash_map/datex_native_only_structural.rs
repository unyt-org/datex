use core::hash::Hash;
use crate::collections::HashMap;
use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::get_datex_type::GetDatexType;

/// If `T` implements [DatexNativeOnlyStructural], then `HashMap<K, V>` also implements [DatexNativeOnlyStructural].
impl<K: DatexNativeOnlyStructural + GetDatexType + Eq + Hash, V: GetDatexType + DatexNativeOnlyStructural> DatexNativeOnlyStructural for HashMap<K, V> {}