use crate::{
    ast::{expressions::NormalExpressionData, spanned::Spanned},
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::list::List,
};

impl ToDatexExpressionData for List {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        NormalExpressionData::List(crate::ast::expressions::List::new(
            self.into_iter()
                .map(|item| item.to_datex_expression_data().with_default_span())
                .collect(),
        ))
    }
}
