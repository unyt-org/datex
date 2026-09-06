use crate::{
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
    },
    values::core_values::decimal::Decimal,
};

impl DatexNativeStructural for Decimal {}
impl DatexNativeOnlyStructural for Decimal {}
