use crate::ast::expressions::DataExpression;

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents the creation of a mutable reference, e.g. `&mut x`.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateMut {
    /// The expression being referenced.
    pub expression: DataExpression,
}
