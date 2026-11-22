//! This module defines the representation for structural list type expressions in the AST.
use crate::{ast::expressions::TypeExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents a structural list type expression in the AST, including the element types.
#[derive(Clone, Debug, PartialEq)]
pub struct StructuralList(pub Vec<TypeExpression>);
