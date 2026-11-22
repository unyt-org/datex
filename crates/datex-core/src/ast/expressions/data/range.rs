use crate::ast::expressions::DatexExpression;

#[derive(Clone, Debug, PartialEq)]
pub struct RangeDeclaration {
    pub start: DatexExpression,
    pub end: DatexExpression,
}
