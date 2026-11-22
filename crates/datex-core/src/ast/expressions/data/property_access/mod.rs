//! This module defines the representation for property access expressions in the AST.
use crate::ast::expressions::DataExpression;

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents a property access expression, e.g. `myStruct.property`.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyAccess {
    /// The base expression on which the property is accessed.
    pub base: DataExpression,
    /// The property being accessed.
    pub property: DataExpression,
}
