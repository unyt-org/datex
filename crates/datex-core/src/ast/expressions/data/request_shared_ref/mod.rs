use crate::shared_values::{PointerAddress, ReferenceMutability};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents the request for a shared reference from a pointer, e.g. `&*ptr`.
#[derive(Clone, Debug, PartialEq)]
pub struct RequestSharedRef {
    /// The mutability of the shared reference.
    pub mutability: ReferenceMutability,
    /// The address of the pointer being dereferenced.
    pub address: PointerAddress,
}
