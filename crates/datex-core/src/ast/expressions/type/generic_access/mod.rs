//! This module defines the representation for generic access type expressions in the AST.
use crate::{ast::expressions::TypeExpression, prelude::*};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents a generic access type expression in the AST, including its base type and the types being accessed.
#[derive(Clone, Debug, PartialEq)]
pub struct GenericAccess {
    /// The base type being accessed.
    pub base: String,
    /// The types being accessed on the base type.
    pub access: Vec<TypeExpression>,
}
