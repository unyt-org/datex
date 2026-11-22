//! This module defines the representation for compile expressions in the AST.
use crate::ast::expressions::DataExpression;

/// Represents a compile expression.
#[derive(Clone, Debug, PartialEq)]
pub struct CompileExpression {
    /// The expression to be compiled.
    pub expression: DataExpression,
}
