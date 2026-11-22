//! This module defines the representation for reference-related expressions in the AST, including shared and mutable references.
use crate::{
    ast::expressions::DatexExpression,
    shared_values::{
        PointerAddress, ReferenceMutability, SharedContainerMutability,
    },
    types::type_definition_with_metadata::LocalReferenceMutability,
};

/// Represents the derivation of a local reference, e.g. `&mut x`.
#[derive(Clone, Debug, PartialEq)]
pub struct DeriveRef {
    /// The mutability of the local reference.
    pub mutability: LocalReferenceMutability,
    /// The expression being referenced.
    pub expression: DatexExpression,
}

/// Represents the derivation of a shared reference, e.g. `&x`.
#[derive(Clone, Debug, PartialEq)]
pub struct DeriveSharedRef {
    /// The mutability of the shared reference.
    pub mutability: ReferenceMutability,
    /// The expression being referenced.
    pub expression: DatexExpression,
}

/// Represents the creation of a mutable reference, e.g. `&mut x`.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateMut {
    /// The expression being referenced.
    pub expression: DatexExpression,
}

/// Represents the request for a shared reference from a pointer, e.g. `&*ptr`.
#[derive(Clone, Debug, PartialEq)]
pub struct RequestSharedRef {
    /// The mutability of the shared reference.
    pub mutability: ReferenceMutability,
    /// The address of the pointer being dereferenced.
    pub address: PointerAddress,
}

/// Represents the creation of a shared container, e.g. `Rc::new(x)`.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateShared {
    /// The mutability of the shared container.
    pub mutability: SharedContainerMutability,
    /// The expression being stored in the shared container.
    pub expression: DatexExpression,
}
