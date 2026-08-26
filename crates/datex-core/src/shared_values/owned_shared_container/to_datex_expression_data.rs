use crate::{
    ast::{
        expressions::{CreateShared, DatexExpressionData},
        spanned::Spanned,
    },
    shared_values::{OwnedSharedContainer, traits::SharedContainerCommon},
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl ToDatexExpressionData for OwnedSharedContainer {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        if self.is_borrowed() {
            DatexExpressionData::OmitRecursive
        } else {
            DatexExpressionData::CreateShared(CreateShared {
                mutability: self.container_mutability(),
                expression: (&*self.value_container())
                    .to_datex_expression_data()
                    .with_default_span(),
            })
        }
    }
}
