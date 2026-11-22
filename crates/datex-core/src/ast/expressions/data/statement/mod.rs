//! This module defines the representation for statement-related expressions in the AST.
use crate::{ast::expressions::DataExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents an unbounded statement, e.g. for a block stream
#[derive(Clone, Debug, PartialEq)]
pub struct UnboundedStatement {
    /// Indicates if this statement is the first in the block stream.
    pub is_first: bool,
    /// Indicates if this statement is the last in the block stream.
    pub is_last: bool,
}

/// Represents a collection of statements, potentially with unbounded context.
#[derive(Clone, Debug, PartialEq)]
pub struct Statements {
    /// The list of statements in this collection.
    pub statements: Vec<DataExpression>,
    /// Indicates if the collection of statements is terminated (by a semicolon)
    pub is_terminated: bool,
    /// The unbounded context associated with this collection of statements, if any.
    pub unbounded: Option<UnboundedStatement>,
}

impl Statements {
    /// Creates a new empty collection of statements, considered terminated.
    pub fn empty() -> Self {
        Statements {
            statements: Vec::new(),
            is_terminated: true,
            unbounded: None,
        }
    }

    /// Creates a new collection of statements that is terminated (by a semicolon).
    pub fn new_terminated(statements: Vec<DataExpression>) -> Self {
        Statements {
            statements,
            is_terminated: true,
            unbounded: None,
        }
    }

    /// Creates a new collection of statements that is unterminated (no semicolon).
    pub fn new_unterminated(statements: Vec<DataExpression>) -> Self {
        Statements {
            statements,
            is_terminated: false,
            unbounded: None,
        }
    }
}
