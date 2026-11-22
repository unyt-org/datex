//! This module defines the representation for type and entity declaration expressions in the AST.
use crate::{
    ast::expressions::{TypeExpression, VariableId},
    prelude::*,
};

/// Represents a type declaration expression in the AST.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeDeclarationExpression {
    /// The unique identifier for this type declaration, if any (set by precompiler)
    pub id: Option<VariableId>,
    /// The name of the type being declared.
    pub name: String, // TODO #614: separate variant from name
    /// The type definition associated with this declaration.
    pub definition: TypeExpression,
    /// Indicates if this type declaration is hoisted.
    pub hoisted: bool, // (set by precompiler)
}

/// Represents an entity declaration expression in the AST.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityDeclarationExpression {
    /// The unique identifier for this entity declaration, if any (set by precompiler)
    pub id: Option<VariableId>,
    /// The name of the entity being declared.
    pub name: String,
    /// The type definition associated with this entity declaration.
    pub definition: TypeExpression,
    /// Indicates if this entity declaration is hoisted (set by precompiler)
    pub hoisted: bool,
}
