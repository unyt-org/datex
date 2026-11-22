//! This module defines the representation for union type expressions in the AST.
use crate::{ast::expressions::TypeExpression, prelude::*};
/// Represents a union type expression in the AST, including the member types.
#[derive(Clone, Debug, PartialEq)]
pub struct Union(pub Vec<TypeExpression>);
