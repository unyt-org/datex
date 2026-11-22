//! This module defines the representation for generic instantiation expressions in the AST.
use crate::{
    ast::expressions::{DataExpression, TypeExpression},
    prelude::*,
};

/// Represents a generic instantiation expression.
#[derive(Clone, Debug, PartialEq)]
pub struct GenericInstantiation {
    /// The base expression being instantiated.
    pub base: DataExpression,
    /// The generic arguments for the instantiation.
    pub generic_arguments: Vec<TypeExpression>,
}
