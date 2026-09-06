use crate::{
    traits::{
        datex_native_only_structural::DatexNativeOnlyStructural,
        datex_native_structural::DatexNativeStructural,
    },
    values::core_values::text::Text,
};

impl DatexNativeStructural for Text {}
impl DatexNativeOnlyStructural for Text {}
