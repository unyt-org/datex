use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::values::core_values::boolean::Boolean;

impl DatexNativeStructural for Boolean {}
impl DatexNativeOnlyStructural for Boolean {}