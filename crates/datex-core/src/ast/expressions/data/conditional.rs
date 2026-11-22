//! This module defines the representation for conditional expressions in the AST.
use crate::ast::expressions::DataExpression;

#[derive(Clone, Debug, PartialEq)]
pub struct Conditional {
    /// The condition of the conditional expression.
    pub condition: DataExpression,
    /// The then branch of the conditional expression.
    pub then_branch: DataExpression,
    /// The else branch of the conditional expression, if any.
    pub else_branch: Option<DataExpression>,
}
