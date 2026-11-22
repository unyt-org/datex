//! This module defines the representation for type variant access expressions in the AST.
use crate::{
    ast::{expressions::TypeExpression, resolved_variable::ResolvedVariable},
    prelude::*,
};

/// Represents a type variant access expression in the AST, including the name, variant, and optional base variable.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeVariantAccess {
    /// The name of the type being accessed.
    pub name: String,
    /// The variant of the type being accessed.
    pub variant: String,
    /// The optional base variable for the type variant access.
    pub base: Option<ResolvedVariable>,
}
