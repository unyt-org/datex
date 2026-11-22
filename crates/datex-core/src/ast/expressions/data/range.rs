//! This module defines the representation for range expressions in the AST.
use crate::ast::expressions::DataExpression;

/// Represents a range expression, e.g. `start..end`.
#[derive(Clone, Debug, PartialEq)]
pub struct RangeDeclaration {
    /// The start expression of the range.
    pub start: DataExpression,
    /// The end expression of the range.
    pub end: DataExpression,
}
