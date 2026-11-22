//! This module defines the representation for conditional expressions in the AST.
use crate::ast::expressions::DatexExpression;

#[derive(Clone, Debug, PartialEq)]
pub struct Conditional {
    /// The condition of the conditional expression.
    pub condition: DatexExpression,
    /// The then branch of the conditional expression.
    pub then_branch: DatexExpression,
    /// The else branch of the conditional expression, if any.
    pub else_branch: Option<DatexExpression>,
}
