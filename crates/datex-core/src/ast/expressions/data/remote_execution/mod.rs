//! This module defines the representation for remote execution expressions in the AST.
use crate::ast::expressions::DataExpression;

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents a remote execution expression, e.g. `@example :: (1 + 1)`.
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteExecution {
    /// The left-hand side expression of the remote execution.
    pub left: DataExpression,
    /// The right-hand side expression of the remote execution.
    pub right: DataExpression,
    /// Internal metadata set by the precompiler, indicates how many variables from the parent scope are accessed inside the remote execution.
    pub injected_variable_count: Option<u32>,
}
