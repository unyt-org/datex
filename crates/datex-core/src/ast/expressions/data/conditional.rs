use crate::ast::expressions::DatexExpression;

#[derive(Clone, Debug, PartialEq)]
pub struct Conditional {
    pub condition: DatexExpression,
    pub then_branch: DatexExpression,
    pub else_branch: Option<DatexExpression>,
}
