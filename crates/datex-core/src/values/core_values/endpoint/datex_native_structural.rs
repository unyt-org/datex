use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::values::core_values::endpoint::Endpoint;

impl DatexNativeStructural for Endpoint {}
impl DatexNativeOnlyStructural for Endpoint {}