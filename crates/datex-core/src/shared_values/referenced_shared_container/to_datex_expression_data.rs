use crate::{
    ast::{
        expressions::{CreateShared, DataExpressionData},
        spanned::Spanned,
    },
    shared_values::{ReferencedSharedContainer, traits::SharedContainerCommon},
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl ToDatexExpressionData for ReferencedSharedContainer {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        if self.is_borrowed() {
            DataExpressionData::OmitRecursive
        } else {
            DataExpressionData::CreateShared(CreateShared {
                mutability: self.container_mutability(),
                expression: (*self.value_container())
                    .to_datex_expression_data()
                    .with_default_span(),
            })
        }
    }
}
