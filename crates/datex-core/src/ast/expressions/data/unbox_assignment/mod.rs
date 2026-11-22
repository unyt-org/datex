use crate::{
    ast::expressions::DataExpression, global::operators::ModificationOperator,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents an assignment to an unbox expression.
#[derive(Clone, Debug, PartialEq)]
pub struct UnboxAssignment {
    /// The operator used for the assignment, if any.
    pub operator: Option<ModificationOperator>,
    /// The unbox expression being assigned to.
    pub unbox_expression: DataExpression,
    /// The expression being assigned.
    pub assigned_expression: DataExpression,
}
