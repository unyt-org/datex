use crate::{
    datex_proxy::{
        DatexProxyType, DatexValueProxySerialize, TryToDatexValueError,
    },
    libs::core::type_id::CoreLibBaseTypeId,
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::{try_clone::TryClone, value_access::ValueAccess},
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

// Note: implemented manually here until amounts are implemented
impl DatexValueProxySerialize for Duration {
    fn try_boxed_to_value(
        self: Box<Self>,
        context: &mut SharedReferencesCache,
    ) -> Result<Value, TryToDatexValueError> {
        todo!()
    }
}
impl DatexProxyType for Duration {
    fn datex_type(context: &mut SharedReferencesCache) -> Type {
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

    fn boxed_to_datex_native_value(
        self: Box<Self>,
        cache: &mut SharedReferencesCache,
    ) -> Value {
        Value::native_boxed(self, cache)
    }
}

impl<'a> AsBorrowed<'a> for Duration {
    fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::native_borrowed(self)
    }
}
impl<'a> AsBorrowedMut<'a> for Duration {
    fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a> {
        BorrowedValueContainerMut::native_borrowed(self)
    }
}
