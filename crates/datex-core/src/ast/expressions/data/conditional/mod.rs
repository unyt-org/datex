//! This module defines the representation for conditional expressions in the AST.
use crate::ast::expressions::NormalExpression;

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

#[derive(Clone, Debug, PartialEq)]
pub struct Conditional {
    /// The condition of the conditional expression.
    pub condition: NormalExpression,
    /// The then branch of the conditional expression.
    pub then_branch: NormalExpression,
    /// The else branch of the conditional expression, if any.
    pub else_branch: Option<NormalExpression>,
}
