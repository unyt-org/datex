use crate::prelude::*;
#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents a root property access expression, e.g. `$.property`.
#[derive(Clone, Debug, PartialEq)]
pub struct RootPropertyAccess {
    /// The name of the property being accessed on the root.
    pub property_name: String,
}
