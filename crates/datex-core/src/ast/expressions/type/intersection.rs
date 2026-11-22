//! This module defines the representation for intersection type expressions in the AST.
use crate::{ast::expressions::TypeExpression, prelude::*};
/// Represents an intersection type expression in the AST, including the types being intersected.
#[derive(Clone, Debug, PartialEq)]
pub struct Intersection(pub Vec<TypeExpression>);
