//! This module defines the representation for variant access expressions in the AST.
use crate::{ast::expressions::ResolvedVariable, prelude::*};

/// Represents a variant access expression in the AST.
#[derive(Clone, Debug, PartialEq)]
pub struct VariantAccess {
    /// The name of the variant being accessed.
    pub name: String,
    /// The specific variant of the enum or type being accessed.
    pub variant: String,
    /// The base variable from which the variant is being accessed.
    pub base: ResolvedVariable,
}
