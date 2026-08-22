use crate::ast::expressions::{DatexExpressionData};
use crate::ast::spanned::Spanned;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::map::Map;
use crate::values::value_container::ValueContainer;

impl ToDatexExpressionData for Map {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Map(crate::ast::expressions::Map::new(
            self.iter()
                .map(|(key, value)| {
                    (
                        ValueContainer::from(key).to_datex_expression_data()
                            .with_default_span(),
                        value.to_datex_expression_data().with_default_span(),
                    )
                })
                .collect(),
        ))
    }
}