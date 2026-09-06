use crate::{
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
    },
    values::core_values::boolean::Boolean,
};

impl DatexNativeStructural for Boolean {}
impl DatexNativeOnlyStructural for Boolean {}
