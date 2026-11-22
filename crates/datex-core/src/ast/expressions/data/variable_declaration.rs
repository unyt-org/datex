//! This module defines the representation for variable declaration expressions in the AST.
use crate::{
    ast::expressions::{
        DataExpression, TypeExpression, VariableId, VariableKind,
    },
    prelude::*,
};

/// Represents a variable declaration expression in the AST.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableDeclaration {
    /// The unique identifier for the variable being declared.
    pub id: Option<VariableId>,
    /// The kind of variable, e.g. [VariableKind::Const] or [VariableKind::Var].
    pub kind: VariableKind,
    /// The name of the variable being declared.
    pub name: String,
    /// The type annotation for the variable, if any (set by the precompiler).
    pub type_annotation: Option<TypeExpression>,
    /// The initialization expression for the variable.
    pub init_expression: DataExpression,
}
