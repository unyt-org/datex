use crate::ast::type_expressions::TypeExpressionData;

pub trait ToTypeExpressionData {
    /// Converts the implementing type into a [TypeExpressionData] representation.
    fn to_type_expression_data(&self) -> TypeExpressionData;
}