//! This module defines the representation for structural list type expressions in the AST.
use crate::{ast::expressions::TypeExpression, prelude::*};

/// Represents a structural list type expression in the AST, including the element types.
#[derive(Clone, Debug, PartialEq)]
pub struct StructuralList(pub Vec<TypeExpression>);
