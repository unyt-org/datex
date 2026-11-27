//! This module defines the representation for unbox expressions in the AST.
use crate::ast::expressions::NormalExpression;

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents an unbox expression in the AST.
#[derive(Clone, Debug, PartialEq)]
pub struct Unbox {
    /// The expression to be unboxed.
    pub expression: NormalExpression,
}
