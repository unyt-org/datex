//! This module defines the representation for remote execution expressions in the AST.
use crate::ast::expressions::DatexExpression;

/// Represents a remote execution expression, e.g. `@example :: (1 + 1)`.
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteExecution {
    /// The left-hand side expression of the remote execution.
    pub left: DatexExpression,
    /// The right-hand side expression of the remote execution.
    pub right: DatexExpression,
    /// Internal metadata set by the precompiler, indicates how many variables from the parent scope are accessed inside the remote execution.
    pub injected_variable_count: Option<u32>,
}
