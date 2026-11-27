//! This module defines the representation for generic instantiation expressions in the AST.
use crate::{
    ast::expressions::{NormalExpression, TypeExpression},
    prelude::*,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents a generic instantiation expression.
#[derive(Clone, Debug, PartialEq)]
pub struct GenericInstantiation {
    /// The base expression being instantiated.
    pub base: NormalExpression,
    /// The generic arguments for the instantiation.
    pub generic_arguments: Vec<TypeExpression>,
}
