use crate::ast::expressions::{DatexExpressionData};
use crate::ast::spanned::Spanned;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::list::List;

impl ToDatexExpressionData for List {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::List(crate::ast::expressions::List::new(
            self.into_iter()
                .map(|item| item.to_datex_expression_data().with_default_span())
                .collect()
        ))
    }
}