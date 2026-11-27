//! This module defines the representation for comparison operation expressions in the AST.
use crate::{
    ast::expressions::NormalExpression, global::operators::ComparisonOperator,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonOperation {
    /// The operator of the comparison operation.
    pub operator: ComparisonOperator,
    /// The left operand of the comparison operation.
    pub left: NormalExpression,
    /// The right operand of the comparison operation.
    pub right: NormalExpression,
}
