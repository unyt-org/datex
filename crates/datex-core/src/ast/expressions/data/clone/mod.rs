//! This module defines the representation for clone expressions in the AST.
use crate::ast::expressions::NormalExpression;

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents a clone expression.
#[derive(Clone, Debug, PartialEq)]
pub struct CloneExpression {
    /// The expression to be cloned.
    pub expression: NormalExpression,
}
