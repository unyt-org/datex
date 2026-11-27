//! This module defines the representation for fixed-size list type expressions in the AST.
use crate::{ast::expressions::TypeExpression, prelude::*};
#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents a fixed-size list type expression in the AST, including its element type and size.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedSizeList {
    /// The type of the elements in the list.
    pub ty: Box<TypeExpression>,
    /// The fixed size of the list.
    pub size: usize,
}
