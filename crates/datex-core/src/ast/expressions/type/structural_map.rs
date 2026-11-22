//! This module defines the representation for structural map type expressions in the AST.
use crate::{ast::expressions::TypeExpression, prelude::*};

/// Represents a structural map type expression in the AST, including the key-value types.
#[derive(Clone, Debug, PartialEq)]
pub struct StructuralMap(pub Vec<(TypeExpression, TypeExpression)>);
