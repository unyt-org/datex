//! This module defines the representation for range expressions in the AST.
use crate::ast::expressions::NormalExpression;

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents a range expression, e.g. `start..end`.
#[derive(Clone, Debug, PartialEq)]
pub struct RangeDeclaration {
    /// The start expression of the range.
    pub start: NormalExpression,
    /// The end expression of the range.
    pub end: NormalExpression,
}
