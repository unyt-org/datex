use crate::{
    ast::expressions::NormalExpression,
    types::type_definition_with_metadata::LocalReferenceMutability,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents the derivation of a local reference, e.g. `&mut x`.
#[derive(Clone, Debug, PartialEq)]
pub struct DeriveRef {
    /// The mutability of the local reference.
    pub mutability: LocalReferenceMutability,
    /// The expression being referenced.
    pub expression: NormalExpression,
}
