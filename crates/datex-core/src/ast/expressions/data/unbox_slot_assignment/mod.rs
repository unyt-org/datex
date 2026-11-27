use crate::{
    ast::expressions::NormalExpression,
    global::{operators::ModificationOperator, stack_index::StackIndex},
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents an assignment to an unbox slot.
#[derive(Clone, Debug, PartialEq)]
pub struct UnboxSlotAssignment {
    /// The operator used for the assignment, if any.
    pub operator: Option<ModificationOperator>,
    /// The stack index of the unbox slot being assigned to.
    pub stack_index: StackIndex,
    /// The expression being assigned.
    pub assigned_expression: NormalExpression,
}
