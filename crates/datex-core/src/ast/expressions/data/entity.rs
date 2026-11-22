//! This module defines the representation for entity value expressions in the AST.
use crate::{
    ast::expressions::DatexExpression, prelude::*,
    shared_values::PointerAddress,
};

/// Represents an entity value expression.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityValueExpression {
    /// The name of the entity.
    pub entity_name: String,
    /// The address of the entity, if any.
    pub entity_address: Option<PointerAddress>,
    /// The value of the entity.
    pub value: DatexExpression,
}
