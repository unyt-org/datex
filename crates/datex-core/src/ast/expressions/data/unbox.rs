use crate::ast::expressions::DatexExpression;

#[derive(Clone, Debug, PartialEq)]
pub struct Unbox {
    pub expression: DatexExpression,
}
