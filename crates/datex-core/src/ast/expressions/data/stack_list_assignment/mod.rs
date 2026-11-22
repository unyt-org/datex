//! This module defines the representation for stack-related expressions in the AST.
use crate::ast::expressions::DataExpression;
#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents an assignment to a list of stack slots (internal used by compiler only)
#[derive(Clone, Debug, PartialEq)]
pub struct StackListAssignment {
    /// The expression being assigned to the list of stack slots.
    pub expression: DataExpression,
}
