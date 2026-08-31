pub mod classification;
pub mod datex_native;

use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::value_access::ValueAccess,
    types::r#type::Type,
    values::{
        borrowed_value_container::{
            AsBorrowed, AsBorrowedMut, BorrowedValueContainer,
        },
        core_values::native::DatexNative,
    },
};
use core::{time::Duration};
use crate::preludes::derive::CoreLibTypeId;
use crate::traits::convert_parts::{FromParts, IntoParts};
use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
use crate::traits::datex_native_structural::DatexNativeStructural;
use crate::traits::get_core_lib_type_id::GetCoreLibTypeId;
use crate::traits::get_datex_type::GetDatexType;

#[cfg(feature = "decompiler")]
mod to_datex_expression_data {
    use crate::{
        ast::expressions::DatexExpressionData,
        traits::to_datex_expression_data::ToDatexExpressionData,
        values::core_values::integer::Integer,
    };
    use core::time::Duration;

    impl ToDatexExpressionData for Duration {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            // TODO: use amount once implemented
            DatexExpressionData::Integer(Integer::from(self.as_millis()))
        }
    }
}

impl ValueAccess for Duration {}
impl FromParts for Duration {}
impl IntoParts for Duration {}
impl GetCoreLibTypeId for Duration {}
impl GetDatexType for Duration {}

impl DatexNativeStructural for Duration {}
impl DatexNativeOnlyStructural for Duration {}