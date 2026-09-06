use crate::{
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
    },
    values::core_values::integer::Integer,
};
impl DatexNativeStructural for Integer {}
impl DatexNativeOnlyStructural for Integer {}
