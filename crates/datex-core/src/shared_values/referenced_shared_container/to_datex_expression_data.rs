use crate::ast::expressions::{CreateShared, DatexExpressionData};
use crate::ast::spanned::Spanned;
use crate::shared_values::ReferencedSharedContainer;
use crate::shared_values::traits::SharedContainerCommon;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;

impl ToDatexExpressionData for ReferencedSharedContainer {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        if self.is_borrowed() {
            DatexExpressionData::OmitRecursive
        } else {
            DatexExpressionData::CreateShared(CreateShared {
                mutability: self.container_mutability(),
                expression: (&*self.value_container()).to_datex_expression_data().with_default_span(),
            })
        }
    }
}