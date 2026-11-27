//! This module defines the representation for entity value expressions in the AST.
use crate::{
    ast::expressions::NormalExpression, prelude::*, shared_values::PointerAddress,
};

#[cfg(feature = "compiler")]
mod ast_node_to_instructions;

/// Represents an entity value expression.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityValueExpression {
    /// The name of the entity.
    pub entity_name: String,
    /// The address of the entity, if any.
    pub entity_address: Option<PointerAddress>,
    /// The value of the entity.
    pub value: NormalExpression,
}
