use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::values::core_values::integer::typed_integer::TypedInteger;

impl DatexNativeStructural for TypedInteger {}
impl DatexNativeOnlyStructural for TypedInteger {}