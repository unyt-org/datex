//! This module defines the representation for unary operation expressions in the AST.
use crate::{
    ast::expressions::DataExpression, global::operators::UnaryOperator,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents a unary operation expression in the AST.
#[derive(Clone, Debug, PartialEq)]
pub struct UnaryOperation {
    /// The unary operator for this operation.
    pub operator: UnaryOperator,
    /// The expression the operator is applied to.
    pub expression: DataExpression,
}
