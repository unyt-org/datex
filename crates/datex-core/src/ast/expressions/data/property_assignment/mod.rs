//! This module defines the representation for assignment expressions in the AST.
use crate::{
    ast::expressions::NormalExpression, global::operators::ModificationOperator,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents an assignment to a property of an object.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyAssignment {
    /// The operator used for the assignment, if any.
    pub operator: Option<ModificationOperator>,
    /// The base object whose property is being assigned.
    pub base: NormalExpression,
    /// The property being assigned to.
    pub property: NormalExpression,
    /// The expression being assigned.
    pub assigned_expression: NormalExpression,
}
