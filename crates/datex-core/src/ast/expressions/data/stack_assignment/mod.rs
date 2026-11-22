//! This module defines the representation for stack-related expressions in the AST.
use crate::{
    ast::expressions::DataExpression, global::stack_index::StackIndex,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents an assignment to a stack slot (internal used by compiler only)
#[derive(Clone, Debug, PartialEq)]
pub struct StackAssignment {
    /// The index of the stack slot being assigned to.
    pub index: StackIndex,
    /// The expression being assigned to the stack slot.
    pub expression: DataExpression,
}
