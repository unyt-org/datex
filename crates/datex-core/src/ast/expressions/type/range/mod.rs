//! This module defines the representation for range type expressions in the AST.
use crate::{ast::expressions::TypeExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents a range type expression in the AST, including its start and end types.
#[derive(Clone, Debug, PartialEq)]
pub struct RangeTypeExpr {
    /// The start type of the range.
    pub start: Box<TypeExpression>,
    /// The end type of the range.
    pub end: Box<TypeExpression>,
}
