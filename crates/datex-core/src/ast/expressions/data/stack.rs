//! This module defines the representation for stack-related expressions in the AST.
use crate::{
    ast::expressions::DatexExpression, global::stack_index::StackIndex,
};

/// Represents an assignment to a stack slot (internal used by compiler only)
#[derive(Clone, Debug, PartialEq)]
pub struct StackAssignment {
    /// The index of the stack slot being assigned to.
    pub index: StackIndex,
    /// The expression being assigned to the stack slot.
    pub expression: DatexExpression,
}

/// Represents an assignment to a list of stack slots (internal used by compiler only)
#[derive(Clone, Debug, PartialEq)]
pub struct StackListAssignment {
    /// The expression being assigned to the list of stack slots.
    pub expression: DatexExpression,
}
