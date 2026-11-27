//! This module defines the representation for reference-related expressions in the AST, including shared and mutable references.
use crate::{
    ast::expressions::NormalExpression, shared_values::ReferenceMutability,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents the derivation of a shared reference, e.g. `&x`.
#[derive(Clone, Debug, PartialEq)]
pub struct DeriveSharedRef {
    /// The mutability of the shared reference.
    pub mutability: ReferenceMutability,
    /// The expression being referenced.
    pub expression: NormalExpression,
}
