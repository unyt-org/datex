use crate::{
    ast::{expressions::DataExpressionData, spanned::Spanned},
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::map::Map,
};

impl ToDatexExpressionData for Map {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::Map(crate::ast::expressions::Map::new(
            self.iter()
                .map(|(key, value)| {
                    (
                        key.to_datex_expression_data().with_default_span(),
                        value.to_datex_expression_data().with_default_span(),
                    )
                })
                .collect(),
        ))
    }
}
