//! This module defines the representation for binary operation expressions in the AST.
use crate::{
    ast::expressions::DatexExpression, global::operators::BinaryOperator,
    types::r#type::Type,
};
/// Represents a binary operation expression.
#[derive(Clone, Debug, PartialEq)]
pub struct BinaryOperation {
    /// The operator of the binary operation.
    pub operator: BinaryOperator,
    /// The left operand of the binary operation.
    pub left: DatexExpression,
    /// The right operand of the binary operation.
    pub right: DatexExpression,
    /// The type of the binary operation, if known.
    pub ty: Option<Type>,
}
