//! This module defines the representation for binary operation expressions in the AST.
use crate::{
    ast::expressions::NormalExpression, global::operators::BinaryOperator,
    types::r#type::Type,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents a binary operation expression.
#[derive(Clone, Debug, PartialEq)]
pub struct BinaryOperation {
    /// The operator of the binary operation.
    pub operator: BinaryOperator,
    /// The left operand of the binary operation.
    pub left: NormalExpression,
    /// The right operand of the binary operation.
    pub right: NormalExpression,
    /// The type of the binary operation, if known.
    pub ty: Option<Type>,
}
