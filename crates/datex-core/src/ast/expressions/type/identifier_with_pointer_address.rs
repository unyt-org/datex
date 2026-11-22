//! This module defines the representation for identifier with pointer address type expressions in the AST.
use crate::{prelude::*, shared_values::PointerAddress};

/// Represents an identifier with a pointer address type expression in the AST, including its name and pointer address.
#[derive(Clone, Debug, PartialEq)]
pub struct IdentifierWithPointerAddress {
    /// The name of the identifier.
    pub name: String,
    /// The pointer address associated with the identifier.
    pub pointer_address: PointerAddress,
}
