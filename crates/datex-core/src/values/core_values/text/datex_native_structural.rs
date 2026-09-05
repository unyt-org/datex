use crate::values::core_values::text::Text;
use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;

impl DatexNativeStructural for Text {}
impl DatexNativeOnlyStructural for Text {}