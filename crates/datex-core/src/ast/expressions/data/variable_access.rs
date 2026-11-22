//! This module defines the representation for variable access expressions in the AST.
use crate::{
    ast::expressions::{ValueAccessType, VariableId},
    prelude::*,
};

/// Represents a variable access expression in the AST.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableAccess {
    /// The unique identifier for the variable being accessed.
    pub id: VariableId,
    /// The name of the variable being accessed.
    pub name: String,
    /// The type of access for this variable, e.g. [ValueAccessType::Borrow] or [ValueAccessType::Clone].
    pub access_type: ValueAccessType,
}
