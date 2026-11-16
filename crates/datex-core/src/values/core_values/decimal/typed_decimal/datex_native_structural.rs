use crate::{
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
    },
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

impl DatexNativeStructural for TypedDecimal {}
impl DatexNativeOnlyStructural for TypedDecimal {}
