use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;

impl DatexNativeStructural for TypedDecimal {}
impl DatexNativeOnlyStructural for TypedDecimal {}