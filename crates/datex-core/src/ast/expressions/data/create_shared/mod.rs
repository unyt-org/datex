use crate::{
    ast::expressions::NormalExpression, shared_values::SharedContainerMutability,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents the creation of a shared container, e.g. `Rc::new(x)`.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateShared {
    /// The mutability of the shared container.
    pub mutability: SharedContainerMutability,
    /// The expression being stored in the shared container.
    pub expression: NormalExpression,
}
