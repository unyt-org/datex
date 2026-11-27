use crate::{
    ast::expressions::{NormalExpression, VariableId},
    global::operators::ModificationOperator,
    prelude::*,
};
#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents an assignment to a variable.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableAssignment {
    /// The ID of the variable being assigned, if known.
    pub id: Option<VariableId>,
    /// The name of the variable being assigned.
    pub name: String,
    /// The operator used for the assignment, if any.
    pub operator: Option<ModificationOperator>,
    /// The expression being assigned to the variable.
    pub expression: NormalExpression,
}
