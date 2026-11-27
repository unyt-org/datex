use crate::{
    ast::{
        expressions::{NormalExpressionData, RangeDeclaration},
        spanned::Spanned,
    },
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::range::Range,
};

impl ToDatexExpressionData for Range {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        NormalExpressionData::Range(RangeDeclaration {
            start: (self.start.to_datex_expression_data().with_default_span()),
            end: (self.end.to_datex_expression_data().with_default_span()),
        })
    }
}
