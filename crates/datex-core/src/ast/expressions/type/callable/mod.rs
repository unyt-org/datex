//! This module defines the representation for callable type expressions in the AST.
use crate::{
    ast::expressions::TypeExpression, prelude::*,
    types::type_definition::callable::CallableKind,
};
#[cfg(feature = "compiler")]
mod ast_node_to_instructions;
/// Represents a callable type expression in the AST, including its parameter types, rest parameter type, return type, and yeet type.
#[derive(Clone, Debug, PartialEq)]
pub struct CallableTypeExpression {
    /// The kind of callable, e.g. [CallableKind::Function] or [CallableKind::Procedure].
    pub kind: CallableKind,
    /// The types of the parameters for the callable, along with their optional names (unnamed / named parameters).
    pub parameter_types: Vec<(Option<String>, TypeExpression)>,
    /// The type of the rest parameter, if any, along with its optional name.
    pub rest_parameter_type: Option<(Option<String>, Box<TypeExpression>)>,
    /// The return type of the callable, if specified.
    pub return_type: Option<Box<TypeExpression>>,
    /// The yeet type of the callable, if specified.
    pub yeet_type: Option<Box<TypeExpression>>,
}
