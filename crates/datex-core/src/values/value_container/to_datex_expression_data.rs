use crate::ast::expressions::{DatexExpressionData, DeriveSharedRef, Statements};
use crate::ast::spanned::Spanned;
use crate::shared_values::SharedContainer;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::value_container::ValueContainer;

impl ToDatexExpressionData for ValueContainer {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        match self {
            ValueContainer::Local(value) => value.to_datex_expression_data(),
            ValueContainer::Shared(shared) => match shared {
                SharedContainer::Referenced(referenced_container) => {
                    DatexExpressionData::DeriveSharedRef(DeriveSharedRef {
                        mutability: referenced_container.reference_mutability(),
                        expression: referenced_container.to_datex_expression_data()
                            .with_default_span(),
                    })
                }
                SharedContainer::Owned(owned_container) => {
                    owned_container.to_datex_expression_data()
                }
            },
        }
    }
}