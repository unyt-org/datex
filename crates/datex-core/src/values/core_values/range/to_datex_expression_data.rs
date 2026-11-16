use crate::{
    ast::{
        expressions::{DatexExpressionData, RangeDeclaration},
        spanned::Spanned,
    },
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::range::Range,
};

impl ToDatexExpressionData for Range {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Range(RangeDeclaration {
            start: (self.start.to_datex_expression_data().with_default_span()),
            end: (self.end.to_datex_expression_data().with_default_span()),
        })
    }
}
