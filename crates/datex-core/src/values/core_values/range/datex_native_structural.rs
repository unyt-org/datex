use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::values::core_values::range::Range;

impl DatexNativeStructural for Range {}
impl DatexNativeOnlyStructural for Range {}