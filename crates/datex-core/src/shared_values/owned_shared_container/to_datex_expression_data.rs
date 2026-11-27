use crate::{
    ast::{
        expressions::{CreateShared, NormalExpressionData},
        spanned::Spanned,
    },
    shared_values::{OwnedSharedContainer, traits::SharedContainerCommon},
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl ToDatexExpressionData for OwnedSharedContainer {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        if self.is_borrowed() {
            NormalExpressionData::OmitRecursive
        } else {
            NormalExpressionData::CreateShared(CreateShared {
                mutability: self.container_mutability(),
                expression: (*self.value_container())
                    .to_datex_expression_data()
                    .with_default_span(),
            })
        }
    }
}
