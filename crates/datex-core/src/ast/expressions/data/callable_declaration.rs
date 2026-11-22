//! This module defines the representation for callable declaration expressions in the AST.
use crate::{
    ast::expressions::{DataExpression, TypeExpression},
    prelude::*,
    types::type_definition::callable::CallableKind,
};

/// Represents a callable declaration expression.
#[derive(Clone, Debug, PartialEq)]
pub struct CallableDeclaration {
    /// The signature of the callable.
    pub signature: CallableSignature,
    /// The body of the callable.
    pub body: DataExpression,
    /// The number of injected variables, if any.
    pub injected_variable_count: Option<u32>,
}

/// Represents the signature of a callable.
#[derive(Clone, Debug, PartialEq)]
pub struct CallableSignature {
    /// The name of the callable, if any.
    pub name: Option<String>,
    /// The kind of the callable.
    pub kind: CallableKind,
    /// Whether the callable requires async.
    pub requires_async: bool,
    /// The parameters of the callable.
    pub parameters: Vec<(String, TypeExpression)>,
    /// The rest parameter of the callable, if any.
    pub rest_parameter: Option<(String, TypeExpression)>,
    /// The return type of the callable, if any.
    pub return_type: Option<TypeExpression>,
    /// The yeet type of the callable, if any.
    pub yeet_type: Option<TypeExpression>,
}
