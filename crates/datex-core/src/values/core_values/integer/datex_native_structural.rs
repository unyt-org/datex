use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::values::core_values::integer::Integer;
impl DatexNativeStructural for Integer {}
impl DatexNativeOnlyStructural for Integer {}