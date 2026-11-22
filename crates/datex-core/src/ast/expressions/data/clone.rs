use crate::ast::expressions::DatexExpression;

#[derive(Clone, Debug, PartialEq)]
pub struct CloneExpression {
    pub expression: DatexExpression,
}
