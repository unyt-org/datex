use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::value_access::ValueAccess,
    types::r#type::Type,
    values::{
        borrowed_value_container::{
            AsBorrowed, AsBorrowedMut, BorrowedValueContainer,
            BorrowedValueContainerMut,
        },
        core_values::native::DatexNative,
        value::Value,
    },
};
use core::{any::Any, time::Duration};
use crate::traits::datex_native_only_structural::DatexNativeOnlyStructural;
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

impl GetDatexType for Duration {
    fn datex_type(_context: &mut SharedReferencesCache) -> Type {
        Type::core(CoreLibBaseTypeId::Any)
    }
}

impl DatexNative for Duration {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn value_datex_type(&self, cache: &mut SharedReferencesCache) -> Type {
        <Self as GetDatexType>::datex_type(cache)
    }
}

impl DatexNativeOnlyStructural for Duration {}

impl<'a> AsBorrowed<'a> for Duration {
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::native_borrowed_only_structural(self)
    }
}
impl<'a> AsBorrowedMut<'a> for Duration {
    fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::native_borrowed_only_structural(self)
    }
}
