//! This module defines the representation for unbox expressions in the AST.
use crate::ast::expressions::DataExpression;

/// Represents an unbox expression in the AST.
#[derive(Clone, Debug, PartialEq)]
pub struct Unbox {
    /// The expression to be unboxed.
    pub expression: DataExpression,
}
