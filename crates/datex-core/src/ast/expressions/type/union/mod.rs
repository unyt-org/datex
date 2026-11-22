//! This module defines the representation for union type expressions in the AST.
use crate::{ast::expressions::TypeExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents a union type expression in the AST, including the member types.
#[derive(Clone, Debug, PartialEq)]
pub struct Union(pub Vec<TypeExpression>);
